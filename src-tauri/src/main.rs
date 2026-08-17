#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod export;
mod state;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Opens the export's merge-request URL in the system browser.
        .plugin(tauri_plugin_opener::init())
        // Update checks run entirely from the webview (single stable channel),
        // so no Rust commands: the JS plugin talks to the endpoint in
        // tauri.conf.json and process::relaunch restarts after install.
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            use tauri::Manager;
            let data_dir = app.path().app_data_dir()?;
            app.manage(state::AppState::from_env(data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::connection_info,
            commands::sync,
            commands::add_preallocation,
            commands::remove_preallocation,
            commands::run_matching,
            commands::check_access,
            commands::commit,
            commands::archive,
            commands::purge
        ])
        .run(tauri::generate_context!())
        .expect("error while running cupid");
}
