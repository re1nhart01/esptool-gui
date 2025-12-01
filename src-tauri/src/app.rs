use std::{
    fs,
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

use crate::config::Config;

pub static ESP_TOOL: OnceLock<Mutex<EspTool>> = OnceLock::new();
const CONFIG_FILENAME: &str = "esp-gui.config.json";

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

    fn get_config(&self) -> (Config, PathBuf) {
        let exe = std::env::current_exe().unwrap();
        let cwd = exe.parent().unwrap().join(CONFIG_FILENAME);
        let data = fs::read_to_string(cwd.clone());

        if let Ok(config_data) = data {
            let config: Config = serde_json::from_str(&config_data).unwrap();
            return (config, cwd);
        }

        return (Config::new(), cwd);
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
        let partition = self.partition_table.clone();

        let config = self.get_config().0;

        let handle = std::thread::spawn(move || {
            println!("{}", curr_esptool.display());
            let mut command = Command::new(curr_esptool)
                .args([
                    "--chip",
                    &config.chip,
                    "-b",
                    &config.baud_rate.to_string(),
                    "--before",
                ])
                .args(&config.before_flags)
                .args(["--after"])
                .args(&config.after_flags)
                .args([
                    "--flash_mode",
                    &config.flash_mode,
                    "--flash_size",
                    &config.flash_size,
                    "--flash_freq",
                    &config.flash_freq,
                    &config.bootloader_start,
                    bootloader.as_str(),
                    &config.partition_start,
                    partition.as_str(),
                    &config.firmware_start,
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

#[tauri::command]
pub fn tauri_get_config_data() -> Config {
    return ESP_TOOL.get().unwrap().lock().unwrap().get_config().0;
}

#[tauri::command]
pub fn tauri_update_config_data(new_cfg: Config) -> bool {
    let esptool = ESP_TOOL.get().unwrap().lock().unwrap();
    let cwd = esptool.get_config().1;
    let rs_path = Path::new(&cwd);

    return esptool.get_config().0.update_config(new_cfg, &rs_path);
}
