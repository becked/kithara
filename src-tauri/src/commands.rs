use crate::catalog::Catalog;
use crate::extractor::{self, ExtractionManager};
use crate::models::{Category, ExtractionState, ExtractionStatus, PlaybackStatus, Sound, UnitType};
use crate::player::PlayerState;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, State};

/// Search for sounds matching the query and filters
#[tauri::command]
pub async fn search_sounds(
    query: String,
    category: Option<String>,
    unit_type: Option<String>,
    catalog: State<'_, Catalog>,
) -> Result<Vec<Sound>, String> {
    catalog.search_sounds(&query, category.as_deref(), unit_type.as_deref())
}

/// Get all available categories
#[tauri::command]
pub async fn get_categories(catalog: State<'_, Catalog>) -> Result<Vec<Category>, String> {
    catalog.get_categories()
}

/// Get all available unit types
#[tauri::command]
pub async fn get_unit_types(catalog: State<'_, Catalog>) -> Result<Vec<UnitType>, String> {
    catalog.get_unit_types()
}

/// Toggle favorite status for a sound
#[tauri::command]
pub async fn toggle_favorite(
    sound_id: String,
    catalog: State<'_, Catalog>,
) -> Result<bool, String> {
    catalog.toggle_favorite(&sound_id)
}

/// Get the count of favorited sounds
#[tauri::command]
pub async fn get_favorites_count(catalog: State<'_, Catalog>) -> Result<u64, String> {
    catalog.count_favorites()
}

/// Get all favorited sounds
#[tauri::command]
pub async fn get_favorites(catalog: State<'_, Catalog>) -> Result<Vec<Sound>, String> {
    catalog.get_favorites()
}

/// Play a sound by its ID and file path
#[tauri::command]
pub async fn play_sound(
    id: String,
    file_path: String,
    player: State<'_, PlayerState>,
) -> Result<(), String> {
    let path = PathBuf::from(&file_path);

    // Validate file exists
    if !path.exists() {
        return Err(format!("Audio file not found: {}", file_path));
    }

    player.play(id, path)
}

/// Stop the currently playing sound
#[tauri::command]
pub async fn stop_sound(player: State<'_, PlayerState>) -> Result<(), String> {
    player.stop()
}

/// Get the current playback status
#[tauri::command]
pub async fn get_playback_status(player: State<'_, PlayerState>) -> Result<PlaybackStatus, String> {
    let status = player.get_status()?;

    Ok(PlaybackStatus {
        is_playing: status.is_playing,
        current_sound_id: status.current_sound_id,
    })
}

/// Get the current extraction status
#[tauri::command]
pub async fn get_extraction_status(
    manager: State<'_, Arc<ExtractionManager>>,
) -> Result<ExtractionStatus, String> {
    Ok(manager.get_status())
}

/// Start the audio extraction process
#[tauri::command]
pub async fn start_extraction(
    app: AppHandle,
    game_path: String,
    manager: State<'_, Arc<ExtractionManager>>,
    _catalog: State<'_, Catalog>,
) -> Result<(), String> {
    // Validate game path
    let game_path = PathBuf::from(&game_path);
    if !game_path.exists() {
        return Err("Game path does not exist".into());
    }

    // Check required files
    let required_files = ["Events.xml", "Audio_Animation.bnk"];
    for file in required_files {
        if !game_path.join(file).exists() {
            return Err(format!("Required file not found: {}", file));
        }
    }

    // Check if already in progress
    let status = manager.get_status();
    if matches!(status.state, ExtractionState::InProgress) {
        return Err("Extraction already in progress".into());
    }

    // Reset state
    manager.reset();

    // Clone for async task
    let manager_clone = Arc::clone(&*manager);

    // Create a new catalog connection for the background task
    let db_path = crate::catalog::get_db_path()?;
    let catalog_for_task = Arc::new(
        Catalog::open(db_path).map_err(|e| format!("Failed to open catalog: {}", e))?
    );

    // Spawn extraction task
    tauri::async_runtime::spawn(async move {
        if let Err(e) = extractor::run_extraction(
            app,
            game_path,
            manager_clone.clone(),
            catalog_for_task,
        )
        .await
        {
            manager_clone.set_error(e);
        }
    });

    Ok(())
}

/// Cancel the current extraction
#[tauri::command]
pub async fn cancel_extraction(
    manager: State<'_, Arc<ExtractionManager>>,
) -> Result<(), String> {
    manager.request_cancel();
    Ok(())
}

/// Clear the cache (database records and sounds folder) for rebuilding
#[tauri::command]
pub async fn clear_cache(
    catalog: State<'_, Catalog>,
    manager: State<'_, Arc<ExtractionManager>>,
) -> Result<(), String> {
    // Clear all database records
    catalog.clear_all()?;

    // Delete the sounds folder
    let cache_dir = extractor::get_cache_dir()?;
    let sounds_dir = cache_dir.join("sounds");
    if sounds_dir.exists() {
        std::fs::remove_dir_all(&sounds_dir)
            .map_err(|e| format!("Failed to delete sounds folder: {}", e))?;
    }

    // Reset extraction state
    manager.reset();

    Ok(())
}

/// Detect the Old World game installation path
#[tauri::command]
pub async fn detect_game_path() -> Result<Option<String>, String> {
    let possible_paths = get_possible_game_paths();

    for path in possible_paths {
        if path.exists() {
            return Ok(Some(path.to_string_lossy().to_string()));
        }
    }

    Ok(None)
}

/// Get platform-specific possible game installation paths
fn get_possible_game_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            // Steam on macOS
            paths.push(
                home.join("Library/Application Support/Steam/steamapps/common/Old World/OldWorld.app/Contents/Resources/Data/StreamingAssets/Audio/GeneratedSoundBanks/Mac")
            );
        }
        // Direct install
        paths.push(PathBuf::from(
            "/Applications/OldWorld.app/Contents/Resources/Data/StreamingAssets/Audio/GeneratedSoundBanks/Mac"
        ));
    }

    #[cfg(target_os = "windows")]
    {
        // Steam on Windows (common locations)
        paths.push(PathBuf::from(
            r"C:\Program Files (x86)\Steam\steamapps\common\Old World\OldWorld_Data\StreamingAssets\Audio\GeneratedSoundBanks\Windows"
        ));
        paths.push(PathBuf::from(
            r"C:\Program Files\Steam\steamapps\common\Old World\OldWorld_Data\StreamingAssets\Audio\GeneratedSoundBanks\Windows"
        ));
        // GOG on Windows
        paths.push(PathBuf::from(
            r"C:\Program Files (x86)\GOG Galaxy\Games\Old World\OldWorld_Data\StreamingAssets\Audio\GeneratedSoundBanks\Windows"
        ));
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(home) = dirs::home_dir() {
            // Steam on Linux
            paths.push(
                home.join(".steam/steam/steamapps/common/Old World/OldWorld_Data/StreamingAssets/Audio/GeneratedSoundBanks/Linux")
            );
            paths.push(
                home.join(".local/share/Steam/steamapps/common/Old World/OldWorld_Data/StreamingAssets/Audio/GeneratedSoundBanks/Linux")
            );
        }
    }

    paths
}

/// Helper module for getting home directory
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf())
    }
}
