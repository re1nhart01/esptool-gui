use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread::JoinHandle,
};

use tauri::{Emitter, Manager};

pub static ESP_TOOL: OnceLock<Mutex<EspTool>> = OnceLock::new();

pub struct EspTool {
    thread_handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
}

impl EspTool {
    pub fn new() -> Self {
        return Self {
            thread_handle: None,
            stop_flag: Arc::new(AtomicBool::new(true)),
        };
    }

    pub fn free_listen_handle(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            handle.join().unwrap();
        }
    }

    pub fn execute_and_listen(&mut self, filename: String, app: tauri::AppHandle) {
        self.stop_flag.store(false, Ordering::Relaxed);

        let stop_flag = self.stop_flag.clone();

        let exe = std::env::current_exe().unwrap();
        let cwd = exe.parent().unwrap();

        app.emit("esp-tool-log", cwd);

        println!("CWD = {:?}", cwd);

        let handle = std::thread::spawn(move || {
            let command = Command::new("sh")
                .arg("stdout.sh")
                .stdout(Stdio::piped())
                .spawn()
                .unwrap();

            let stdout = command.stdout.unwrap();

            let buff = BufReader::new(stdout);

            for line in buff.lines() {
                if stop_flag.load(Ordering::Relaxed) {
                    break;
                }

                let _ = app.emit("esp-tool-log", line.unwrap());
            }
        });

        self.thread_handle = Some(handle)
    }
}

#[tauri::command]
pub fn tauri_execute_and_listen(app: tauri::AppHandle, filename: String) {
    ESP_TOOL
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .execute_and_listen(filename, app);
}

#[tauri::command]
pub fn tauri_free_listen_handle() {
    ESP_TOOL.get().unwrap().lock().unwrap().free_listen_handle();
}
