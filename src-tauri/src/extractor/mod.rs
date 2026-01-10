//! Audio extraction orchestrator.
//! Manages extraction state and coordinates parsing, extraction, and conversion.

pub mod bnk_parser;
pub mod converter;
pub mod metadata;

use crate::catalog::Catalog;
use crate::models::{ExtractionState, ExtractionStatus, Sound};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::AppHandle;

/// Thread-safe extraction state for Tauri managed state
pub struct ExtractionManager {
    status: Mutex<ExtractionStatus>,
    cancel_flag: Mutex<bool>,
}

impl ExtractionManager {
    pub fn new() -> Self {
        Self {
            status: Mutex::new(ExtractionStatus::default()),
            cancel_flag: Mutex::new(false),
        }
    }

    pub fn get_status(&self) -> ExtractionStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn update_status(
        &self,
        state: ExtractionState,
        progress: f32,
        current_file: Option<String>,
    ) {
        let mut status = self.status.lock().unwrap();
        status.state = state;
        status.progress = progress;
        status.current_file = current_file;
    }

    pub fn set_error(&self, error: String) {
        let mut status = self.status.lock().unwrap();
        status.state = ExtractionState::Error;
        status.error = Some(error);
    }

    pub fn request_cancel(&self) {
        *self.cancel_flag.lock().unwrap() = true;
    }

    pub fn is_cancelled(&self) -> bool {
        *self.cancel_flag.lock().unwrap()
    }

    pub fn reset(&self) {
        *self.status.lock().unwrap() = ExtractionStatus::default();
        *self.cancel_flag.lock().unwrap() = false;
    }
}

impl Default for ExtractionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the cache directory for storing extracted sounds
pub fn get_cache_dir() -> Result<PathBuf, String> {
    let proj_dirs = directories::ProjectDirs::from("com", "kithara", "app")
        .ok_or_else(|| "Failed to determine cache directory".to_string())?;
    Ok(proj_dirs.data_dir().to_path_buf())
}

/// Main extraction entry point
pub async fn run_extraction(
    app: AppHandle,
    game_path: PathBuf,
    manager: Arc<ExtractionManager>,
    catalog: Arc<Catalog>,
) -> Result<(), String> {
    manager.update_status(
        ExtractionState::InProgress,
        0.0,
        Some("Parsing metadata...".into()),
    );

    // Step 1: Parse soundbank XML files to get WEM file ID -> metadata mapping
    let xml_files = vec![
        ("Audio_Animation.xml", "Audio_Animation.bnk"),
        ("Audio_2D.xml", "Audio_2D.bnk"),
        ("Audio_3D.xml", "Audio_3D.bnk"),
    ];

    let mut file_metadata = std::collections::HashMap::new();
    for (xml_name, _) in &xml_files {
        let xml_path = game_path.join(xml_name);
        if xml_path.exists() {
            match metadata::parse_soundbank_xml(&xml_path) {
                Ok(files) => {
                    println!("Parsed {} file entries from {}", files.len(), xml_name);
                    file_metadata.extend(files);
                }
                Err(e) => {
                    println!("Warning: Failed to parse {}: {}", xml_name, e);
                }
            }
        }
    }
    println!("Total file metadata entries: {}", file_metadata.len());

    manager.update_status(
        ExtractionState::InProgress,
        0.05,
        Some("Parsing soundbanks...".into()),
    );

    // Step 2: Parse BNK files and extract WEM data
    let bnk_files = vec!["Audio_Animation.bnk", "Audio_2D.bnk", "Audio_3D.bnk"];

    let mut all_wem_entries = Vec::new();
    for bnk_name in &bnk_files {
        if manager.is_cancelled() {
            return Err("Extraction cancelled".into());
        }

        let bnk_path = game_path.join(bnk_name);
        if !bnk_path.exists() {
            println!("Skipping missing BNK: {}", bnk_name);
            continue;
        }

        println!("Parsing {}...", bnk_name);
        let entries = bnk_parser::parse_bnk(&bnk_path)?;
        println!("  Found {} WEM entries", entries.len());
        all_wem_entries.extend(entries);
    }

    if all_wem_entries.is_empty() {
        return Err("No audio files found in soundbanks".into());
    }

    println!("Total WEM entries: {}", all_wem_entries.len());

    manager.update_status(
        ExtractionState::InProgress,
        0.10,
        Some("Extracting audio...".into()),
    );

    // Step 3: Setup directories
    let cache_dir = get_cache_dir()?;
    let temp_dir = cache_dir.join("temp");
    let sounds_dir = cache_dir.join("sounds");

    std::fs::create_dir_all(&temp_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;
    std::fs::create_dir_all(&sounds_dir)
        .map_err(|e| format!("Failed to create sounds dir: {}", e))?;

    // Step 4: Extract and convert each WEM file
    let total = all_wem_entries.len();
    let mut processed = 0;
    let mut successful = 0;
    let mut skipped_no_metadata = 0;

    for entry in all_wem_entries {
        if manager.is_cancelled() {
            // Cleanup temp files
            let _ = std::fs::remove_dir_all(&temp_dir);
            return Err("Extraction cancelled".into());
        }

        // Try to find matching file metadata from soundbank XML
        let file_info = file_metadata.get(&entry.file_id);
        if file_info.is_none() {
            // Skip files without metadata (shouldn't happen often)
            skipped_no_metadata += 1;
            processed += 1;
            continue;
        }
        let file_info = file_info.unwrap();

        // Extract WEM bytes to temp file
        let wem_path = temp_dir.join(format!("{}.wem", entry.file_id));
        if let Err(e) = bnk_parser::extract_wem_bytes(&entry, &wem_path) {
            eprintln!("Failed to extract WEM {}: {}", entry.file_id, e);
            processed += 1;
            continue;
        }

        // Build output path based on file metadata
        let (category, unit_type, subcategory) = metadata::parse_short_name(&file_info.short_name);
        let output_subdir = if let Some(ref unit) = unit_type {
            sounds_dir.join(&category).join(unit.to_lowercase())
        } else {
            sounds_dir.join(&category)
        };
        std::fs::create_dir_all(&output_subdir)
            .map_err(|e| format!("Failed to create output dir: {}", e))?;

        // Generate clean filename from file ID and short name
        let filename = format!("{}_{}", entry.file_id, sanitize_filename(&file_info.short_name));
        let output_path = output_subdir.join(format!("{}.ogg", filename));

        // Convert WEM -> WAV -> OGG
        match converter::convert_wem_to_ogg(&app, &wem_path, &output_path).await {
            Ok(_) => {
                // Insert into catalog
                let sound = Sound {
                    id: format!("{}", entry.file_id),
                    event_name: file_info.short_name.clone(),
                    display_name: metadata::format_short_name_display(&file_info.short_name),
                    category: category.clone(),
                    unit_type: unit_type.clone(),
                    subcategory: subcategory.clone(),
                    duration: 0.0, // Duration not available from file metadata
                    file_path: output_path.to_string_lossy().to_string(),
                    tags: build_tags(&file_info.short_name, &category, unit_type.as_deref()),
                    is_favorite: false,
                };

                if let Err(e) = catalog.insert_sound(&sound) {
                    eprintln!("Failed to insert sound into catalog: {}", e);
                } else {
                    successful += 1;
                }
            }
            Err(e) => {
                eprintln!("Failed to convert {}: {}", file_info.short_name, e);
            }
        }

        // Cleanup temp WEM
        let _ = std::fs::remove_file(&wem_path);

        processed += 1;
        let progress = 0.10 + (processed as f32 / total as f32) * 0.85;
        manager.update_status(
            ExtractionState::InProgress,
            progress,
            Some(file_info.short_name.clone()),
        );
    }

    if skipped_no_metadata > 0 {
        println!("Skipped {} files without metadata", skipped_no_metadata);
    }

    // Cleanup temp directory
    let _ = std::fs::remove_dir_all(&temp_dir);

    println!(
        "Extraction complete: {} sounds extracted successfully",
        successful
    );

    manager.update_status(ExtractionState::Complete, 1.0, None);
    Ok(())
}

/// Sanitize a filename by removing/replacing invalid characters
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Build searchable tags from event metadata
fn build_tags(event_name: &str, category: &str, unit_type: Option<&str>) -> Vec<String> {
    let mut tags = vec![category.to_string()];

    if let Some(unit) = unit_type {
        tags.push(unit.to_lowercase());
    }

    // Add action keywords as tags
    let keywords = ["attack", "death", "hit", "run", "vocal", "impact", "step"];
    let name_lower = event_name.to_lowercase();
    for keyword in keywords {
        if name_lower.contains(keyword) {
            tags.push(keyword.to_string());
        }
    }

    tags
}
