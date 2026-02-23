#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod config;
mod constants;
mod zipper;
mod monitor;
mod serial;

use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            app::tauri_execute_and_listen,
            app::tauri_free_listen_handle,
            app::tauri_add_file_into_scope,
            app::tauri_get_config_data,
            app::tauri_update_config_data,
            app::tauri_monitor_start,
            app::tauri_monitor_stop,
            app::tauri_get_serial_ports,
            app::tauri_set_selected_port,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    let mut api = app::EspTool::new();
    let monitor = monitor::Monitor::new();
    api.initial_create_config_file();
    let _ = app::ESP_TOOL.set(Mutex::new(api));
    let _ = app::ESP_MONITOR.set(Mutex::new(monitor));

    run();
}
