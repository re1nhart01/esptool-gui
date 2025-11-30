#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;

use std::sync::Mutex;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            app::tauri_execute_and_listen,
            app::tauri_free_listen_handle,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    let api = app::EspTool::new();
    let _ = app::ESP_TOOL.set(Mutex::new(api));

    run()
}
