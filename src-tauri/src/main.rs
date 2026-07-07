#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::sync,
            commands::add_appeal,
            commands::remove_appeal,
            commands::run_matching,
            commands::commit,
            commands::archive,
            commands::purge
        ])
        .run(tauri::generate_context!())
        .expect("error while running cupid");
}
