use crate::models::{Category, ExtractionStatus, Sound, UnitType};
use std::path::PathBuf;

/// Search for sounds matching the query and filters
#[tauri::command]
pub async fn search_sounds(
    query: String,
    category: Option<String>,
    unit_type: Option<String>,
) -> Result<Vec<Sound>, String> {
    // TODO: Implement actual search from SQLite catalog
    Ok(vec![])
}

/// Get all available categories
#[tauri::command]
pub async fn get_categories() -> Result<Vec<Category>, String> {
    // TODO: Implement fetching categories from catalog
    Ok(vec![
        Category {
            id: "unit_attack".to_string(),
            name: "Unit Attacks".to_string(),
            count: 0,
        },
        Category {
            id: "unit_death".to_string(),
            name: "Unit Deaths".to_string(),
            count: 0,
        },
        Category {
            id: "unit_hit".to_string(),
            name: "Unit Hits".to_string(),
            count: 0,
        },
        Category {
            id: "combat".to_string(),
            name: "Combat".to_string(),
            count: 0,
        },
        Category {
            id: "story_event".to_string(),
            name: "Story Events".to_string(),
            count: 0,
        },
    ])
}

/// Get all available unit types
#[tauri::command]
pub async fn get_unit_types() -> Result<Vec<UnitType>, String> {
    // TODO: Implement fetching unit types from catalog
    Ok(vec![])
}

/// Play a sound by its ID
#[tauri::command]
pub async fn play_sound(id: String) -> Result<(), String> {
    // TODO: Implement audio playback with rodio
    println!("Playing sound: {}", id);
    Ok(())
}

/// Stop the currently playing sound
#[tauri::command]
pub async fn stop_sound() -> Result<(), String> {
    // TODO: Implement stop playback
    Ok(())
}

/// Get the current extraction status
#[tauri::command]
pub async fn get_extraction_status() -> Result<ExtractionStatus, String> {
    // TODO: Implement extraction status tracking
    Ok(ExtractionStatus::default())
}

/// Start the audio extraction process
#[tauri::command]
pub async fn start_extraction() -> Result<(), String> {
    // TODO: Implement extraction process
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
