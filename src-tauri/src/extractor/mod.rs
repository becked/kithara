//! Audio extraction orchestrator.
//! Manages extraction state and coordinates parsing, extraction, and conversion.

pub mod bnk_parser;
pub mod converter;
pub mod metadata;

use crate::catalog::Catalog;
use crate::models::{ExtractionState, ExtractionStatus, MusicTrack, Sound};
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
    include_music: bool,
) -> Result<(), String> {
    manager.update_status(
        ExtractionState::InProgress,
        0.0,
        Some("Parsing metadata...".into()),
    );

    // Step 1: Discover soundbanks and parse XML metadata
    let soundbank_pairs = metadata::discover_soundbanks(&game_path)?;
    if soundbank_pairs.is_empty() {
        return Err("No soundbanks with embedded audio found in game directory".into());
    }
    println!(
        "Discovered {} soundbanks: {:?}",
        soundbank_pairs.len(),
        soundbank_pairs.iter().map(|(_, b)| b.as_str()).collect::<Vec<_>>()
    );

    let mut file_metadata = std::collections::HashMap::new();
    let mut music_file_ids: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for (xml_name, _) in &soundbank_pairs {
        let xml_path = game_path.join(xml_name);
        if xml_path.exists() {
            // Check if this is a music bank (contains ReferencedStreamedFiles)
            let content = std::fs::read_to_string(&xml_path).unwrap_or_default();
            let is_music_bank = content.contains("ReferencedStreamedFiles");

            match metadata::parse_soundbank_xml(&xml_path) {
                Ok(files) => {
                    println!("Parsed {} file entries from {}{}", files.len(), xml_name,
                        if is_music_bank { " (music bank)" } else { "" });
                    if is_music_bank {
                        music_file_ids.extend(files.keys());
                    }
                    file_metadata.extend(files);
                }
                Err(e) => {
                    println!("Warning: Failed to parse {}: {}", xml_name, e);
                }
            }
        }
    }
    println!("Total file metadata entries: {}", file_metadata.len());

    // Build dynamic unit list from Event ObjectPaths
    let animation_xml = game_path.join("Audio_Animation.xml");
    let known_units = if animation_xml.exists() {
        match metadata::parse_event_unit_names(&animation_xml) {
            Ok(units) => {
                println!("Discovered {} unit types from Events", units.len());
                units
            }
            Err(e) => {
                println!("Warning: Failed to parse unit names: {}. Unit categorization will be limited.", e);
                Vec::new()
            }
        }
    } else {
        println!("Warning: Audio_Animation.xml not found. Unit categorization will be limited.");
        Vec::new()
    };

    // Progress allocation depends on whether music is included
    let bnk_start = 0.10;
    let bnk_end = if include_music { 0.50 } else { 0.95 };
    let music_start = 0.50;
    let music_end = 1.0;

    manager.update_status(
        ExtractionState::InProgress,
        0.05,
        Some("Parsing soundbanks...".into()),
    );

    // Step 2: Parse BNK files and extract WEM data
    let bnk_files: Vec<&str> = soundbank_pairs.iter().map(|(_, b)| b.as_str()).collect();

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

        // Skip sounds matching exclusion patterns
        if is_excluded(&file_info.short_name, include_music) {
            processed += 1;
            continue;
        }

        // Check if this is a music file (by name convention or source bank)
        let is_music = is_music_file(&file_info.short_name) || music_file_ids.contains(&entry.file_id);

        // Build output path based on file metadata
        let output_subdir = if is_music {
            // Music goes to sounds/music/
            sounds_dir.join("music")
        } else {
            let (category, unit_type, _) = metadata::parse_short_name(&file_info.short_name, &known_units);
            if let Some(ref unit) = unit_type {
                sounds_dir.join(&category).join(unit.to_lowercase())
            } else {
                sounds_dir.join(&category)
            }
        };
        std::fs::create_dir_all(&output_subdir)
            .map_err(|e| format!("Failed to create output dir: {}", e))?;

        // Generate clean filename from file ID and short name
        let filename = format!("{}_{}", entry.file_id, sanitize_filename(&file_info.short_name));
        let output_path = output_subdir.join(format!("{}.ogg", filename));

        // Skip if already converted
        if output_path.exists() {
            processed += 1;
            let progress = bnk_start + (processed as f32 / total as f32) * (bnk_end - bnk_start);
            manager.update_status(
                ExtractionState::InProgress,
                progress,
                Some(file_info.short_name.clone()),
            );
            continue;
        }

        // Extract WEM bytes to temp file
        let wem_path = temp_dir.join(format!("{}.wem", entry.file_id));
        if let Err(e) = bnk_parser::extract_wem_bytes(&entry, &wem_path) {
            eprintln!("Failed to extract WEM {}: {}", entry.file_id, e);
            processed += 1;
            continue;
        }

        // Convert WEM -> WAV -> OGG
        match converter::convert_wem_to_ogg(&app, &wem_path, &output_path).await {
            Ok(_) => {
                if is_music {
                    // Get duration from the converted file
                    let duration_secs = converter::get_audio_duration(&output_path)
                        .await
                        .unwrap_or(0.0);

                    // Insert into music_tracks table
                    let track = MusicTrack {
                        id: format!("{}", entry.file_id),
                        title: metadata::format_music_title(&file_info.short_name),
                        file_path: output_path.to_string_lossy().to_string(),
                        duration_secs,
                    };

                    if let Err(e) = catalog.insert_music_track(&track) {
                        eprintln!("Failed to insert music track into catalog: {}", e);
                    } else {
                        successful += 1;
                    }
                } else {
                    // Insert into sounds table
                    let (category, unit_type, subcategory) = metadata::parse_short_name(&file_info.short_name, &known_units);
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
            }
            Err(e) => {
                eprintln!("Failed to convert {}: {}", file_info.short_name, e);
            }
        }

        // Cleanup temp WEM
        let _ = std::fs::remove_file(&wem_path);

        processed += 1;
        let progress = bnk_start + (processed as f32 / total as f32) * (bnk_end - bnk_start);
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

    // Step 5: Extract streamed music files if requested
    if include_music {
        manager.update_status(
            ExtractionState::InProgress,
            music_start,
            Some("Extracting music tracks...".into()),
        );

        let music_result = extract_streamed_music(
            &app,
            &game_path,
            &sounds_dir,
            &catalog,
            &manager,
            music_start,
            music_end,
        ).await;

        if let Err(e) = music_result {
            eprintln!("Warning: Music extraction failed: {}", e);
        }
    }

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

/// Exclusion patterns for content filtering (case-insensitive substrings)
const EXCLUSION_PATTERNS: &[&str] = &[];

/// Check if a file name indicates a music track
fn is_music_file(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with("mus.") || lower.starts_with("bgm.") || lower.contains("music_")
}

/// Check if a sound name matches any exclusion pattern (case-insensitive substring)
/// If include_music is false, music files are also excluded
fn is_excluded(name: &str, include_music: bool) -> bool {
    let name_lower = name.to_lowercase();

    // Always exclude unreleased content
    if EXCLUSION_PATTERNS.iter().any(|p| name_lower.contains(p)) {
        return true;
    }

    // Exclude music unless opted in
    if !include_music && is_music_file(&name_lower) {
        return true;
    }

    false
}

/// Extract streamed music files (loose WEM files referenced in SoundbanksInfo.xml)
async fn extract_streamed_music(
    app: &AppHandle,
    game_path: &PathBuf,
    sounds_dir: &PathBuf,
    catalog: &Arc<Catalog>,
    manager: &Arc<ExtractionManager>,
    progress_start: f32,
    progress_end: f32,
) -> Result<(), String> {
    // Parse SoundbanksInfo.xml to get streamed file mappings
    let soundbanks_info_path = game_path.join("SoundbanksInfo.xml");
    if !soundbanks_info_path.exists() {
        return Err("SoundbanksInfo.xml not found".into());
    }

    let streamed_files = metadata::parse_soundbanks_info_xml(&soundbanks_info_path)?;
    println!("Found {} streamed music files", streamed_files.len());

    if streamed_files.is_empty() {
        return Ok(());
    }

    // Create music output directory
    let music_dir = sounds_dir.join("music");
    std::fs::create_dir_all(&music_dir)
        .map_err(|e| format!("Failed to create music dir: {}", e))?;

    let total = streamed_files.len();
    let mut processed = 0;
    let mut successful = 0;

    for (file_id, file_info) in &streamed_files {
        if manager.is_cancelled() {
            return Err("Extraction cancelled".into());
        }

        // The WEM file should exist as {file_id}.wem in the game directory
        let wem_path = game_path.join(format!("{}.wem", file_id));
        if !wem_path.exists() {
            processed += 1;
            continue;
        }

        // Generate output filename
        let title = metadata::format_streamed_music_title(&file_info.short_name);
        let safe_title = sanitize_filename(&title);
        let output_path = music_dir.join(format!("{}_{}.ogg", file_id, safe_title));

        // Skip if already converted
        if output_path.exists() {
            processed += 1;
            successful += 1;
            let progress = progress_start + (processed as f32 / total as f32) * (progress_end - progress_start);
            manager.update_status(
                ExtractionState::InProgress,
                progress,
                Some(format!("Music: {} (cached)", title)),
            );
            continue;
        }

        // Convert WEM -> OGG
        match converter::convert_wem_to_ogg(app, &wem_path, &output_path).await {
            Ok(_) => {
                // Get duration from the converted file
                let duration_secs = converter::get_audio_duration(&output_path)
                    .await
                    .unwrap_or(0.0);

                // Insert into music_tracks table
                let track = MusicTrack {
                    id: format!("{}", file_id),
                    title: title.clone(),
                    file_path: output_path.to_string_lossy().to_string(),
                    duration_secs,
                };

                if let Err(e) = catalog.insert_music_track(&track) {
                    eprintln!("Failed to insert music track into catalog: {}", e);
                } else {
                    successful += 1;
                }
            }
            Err(e) => {
                eprintln!("Failed to convert music file {}: {}", file_info.short_name, e);
            }
        }

        processed += 1;
        let progress = progress_start + (processed as f32 / total as f32) * (progress_end - progress_start);
        manager.update_status(
            ExtractionState::InProgress,
            progress,
            Some(format!("Music: {}", title)),
        );
    }

    println!("Music extraction complete: {} tracks extracted", successful);
    Ok(())
}
