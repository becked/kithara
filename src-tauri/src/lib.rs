mod catalog;
mod commands;
mod models;
mod player;

use catalog::{get_db_path, Catalog};
use player::create_player_state;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let player_state = create_player_state().expect("Failed to initialize audio player");

    // Initialize catalog database
    let db_path = get_db_path().expect("Failed to determine database path");
    println!("Database path: {:?}", db_path);
    let catalog = Catalog::open(db_path).expect("Failed to initialize catalog");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(player_state)
        .manage(catalog)
        .invoke_handler(tauri::generate_handler![
            commands::search_sounds,
            commands::get_categories,
            commands::get_unit_types,
            commands::play_sound,
            commands::stop_sound,
            commands::get_playback_status,
            commands::get_extraction_status,
            commands::start_extraction,
            commands::detect_game_path,
        ])
        .setup(|app| {
            // Seed test sounds if database is empty
            let catalog = app.state::<Catalog>();
            if let Ok(count) = catalog.count_sounds() {
                if count == 0 {
                    println!("Database empty, seeding test sounds...");
                    if let Err(e) = seed_test_sounds(app) {
                        eprintln!("Warning: Failed to seed test sounds: {}", e);
                    }
                } else {
                    println!("Database has {} sounds", count);
                }
            }

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Seeds the database with test sounds from bundled resources.
fn seed_test_sounds(app: &tauri::App) -> Result<(), String> {
    let catalog = app.state::<Catalog>();
    let resource_path = app
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let test_sounds = vec![
        models::Sound {
            id: "test-short".to_string(),
            event_name: "Test_Short_Sound".to_string(),
            display_name: "Short Test (1s)".to_string(),
            category: "test".to_string(),
            unit_type: None,
            subcategory: "test".to_string(),
            duration: 1.0,
            file_path: resource_path
                .join("resources/test-sounds/test-short.ogg")
                .to_string_lossy()
                .to_string(),
            tags: vec!["test".to_string(), "short".to_string()],
        },
        models::Sound {
            id: "test-medium".to_string(),
            event_name: "Test_Medium_Sound".to_string(),
            display_name: "Medium Test (3s)".to_string(),
            category: "test".to_string(),
            unit_type: Some("Warrior".to_string()),
            subcategory: "test".to_string(),
            duration: 3.0,
            file_path: resource_path
                .join("resources/test-sounds/test-medium.ogg")
                .to_string_lossy()
                .to_string(),
            tags: vec!["test".to_string(), "medium".to_string()],
        },
        models::Sound {
            id: "test-long".to_string(),
            event_name: "Test_Long_Sound".to_string(),
            display_name: "Long Test (10s)".to_string(),
            category: "test".to_string(),
            unit_type: Some("Archer".to_string()),
            subcategory: "test".to_string(),
            duration: 10.0,
            file_path: resource_path
                .join("resources/test-sounds/test-long.ogg")
                .to_string_lossy()
                .to_string(),
            tags: vec!["test".to_string(), "long".to_string()],
        },
    ];

    for sound in &test_sounds {
        catalog.insert_sound(sound)?;
    }

    println!("Seeded {} test sounds", test_sounds.len());
    Ok(())
}
