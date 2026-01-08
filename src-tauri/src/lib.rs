mod commands;
mod models;
mod player;

use player::create_player_state;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let player_state = create_player_state().expect("Failed to initialize audio player");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(player_state)
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
