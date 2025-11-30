use std::{
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread::JoinHandle,
};

use tauri::Emitter;

pub static ESP_TOOL: OnceLock<Mutex<EspTool>> = OnceLock::new();

pub struct EspTool {
    thread_handle: Option<JoinHandle<()>>,
    bootloader_path: String,
    firmware_path: String,
    partition_table: String,
    stop_flag: Arc<AtomicBool>,
}

impl EspTool {
    pub fn new() -> Self {
        return Self {
            thread_handle: None,
            stop_flag: Arc::new(AtomicBool::new(true)),
            bootloader_path: String::from(""),
            firmware_path: String::from(""),
            partition_table: String::from(""),
        };
    }

    pub fn get_current_esptool(&self) -> String {
        let curr_os = std::env::consts::OS.to_string();
        let curr_arch = std::env::consts::ARCH.to_string();

        return format!("esptool-{}-{}", curr_os, curr_arch);
    }

    pub fn add_file_into_scope(&mut self, file_type: String, filename: String) -> bool {
        if file_type == "Bootloader" {
            self.bootloader_path = filename;
        } else if file_type == "Partition Table" {
            self.partition_table = filename;
        } else if file_type == "Firmware" {
            self.firmware_path = filename;
        }

        return true;
    }

    pub fn free_listen_handle(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            handle.join().unwrap();
        }
    }

    fn get_esptool_executor(&self) -> PathBuf {
        let exe = std::env::current_exe().unwrap();
        let cwd = exe.parent().unwrap();

        return cwd
            .join("vendor")
            .join(self.get_current_esptool())
            .join(if cfg!(windows) {
                "esptool.exe"
            } else {
                "esptool"
            });
    }

    pub fn execute_and_listen(&mut self, app: tauri::AppHandle) {
        if self.bootloader_path.is_empty()
            || self.partition_table.is_empty()
            || self.firmware_path.is_empty()
        {
            return;
        }

        self.stop_flag.store(false, Ordering::Relaxed);
        let stop_flag = self.stop_flag.clone();

        let curr_esptool = self.get_esptool_executor();

        let bootloader = self.bootloader_path.clone();
        let firmware = self.firmware_path.clone();
        let partition = self.bootloader_path.clone();

        let handle = std::thread::spawn(move || {
            println!("{}", curr_esptool.display());
            let mut command = Command::new(curr_esptool)
                .args([
                    "--chip",
                    "esp32s3",
                    "-b",
                    "460800",
                    "--before",
                    "default-reset",
                    "--after",
                    "hard-reset",
                    "write-flash",
                    "--flash-mode",
                    "dio",
                    "--flash-freq",
                    "80m",
                    "--flash-size",
                    "8MB",
                    "0x0",
                    bootloader.as_str(),
                    "0x8000",
                    partition.as_str(),
                    "0x10000",
                    firmware.as_str(),
                ])
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to start esptool");

            let stdout = command.stdout.take().unwrap();
            let stderr = command.stderr.take().unwrap();

            let buff = BufReader::new(stdout);
            let buff_err = BufReader::new(stderr);

            let chain = buff.chain(buff_err);

            for line in chain.lines() {
                if stop_flag.load(Ordering::Relaxed) {
                    let _ = command.kill();
                    break;
                }

                let _ = app.emit("esp-tool-log", line.unwrap());
            }
        });

        self.thread_handle = Some(handle)
    }
}

#[tauri::command]
pub fn tauri_execute_and_listen(app: tauri::AppHandle) {
    ESP_TOOL
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .execute_and_listen(app);
}

#[tauri::command]
pub fn tauri_free_listen_handle() {
    ESP_TOOL.get().unwrap().lock().unwrap().free_listen_handle();
}

#[tauri::command]
pub fn tauri_add_file_into_scope(file_type: String, filename: String) -> bool {
    println!("{}", filename);
    ESP_TOOL
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .add_file_into_scope(file_type, filename);
    return true;
}
