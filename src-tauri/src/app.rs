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

use tauri::{Emitter};

use crate::{config::Config, constants, monitor::Monitor, serial::get_list_serial_ports, zipper::Zipper};

pub static ESP_TOOL: OnceLock<Mutex<EspTool>> = OnceLock::new();
pub static ESP_MONITOR: OnceLock<Mutex<Monitor>> = OnceLock::new();

const CONFIG_FILENAME: &str = "esp-gui.config.json";

#[derive(Clone, Debug)]
struct EspToolState {
    bootloader_path: String,
    firmware_path: String,
    partition_table_path: String,
    archive_path: String,
    storage_path: String,
    ota_data_initial_path: String,
    unpacked_dir: String,
    firmware_elf: String,
    selected_port: String,
}

pub struct EspTool {
    thread_handle: Option<JoinHandle<()>>,
    state: EspToolState,
    stop_flag: Arc<AtomicBool>,
}

impl EspTool {
    pub fn new() -> Self {
        return Self {
            thread_handle: None,
            stop_flag: Arc::new(AtomicBool::new(true)),
            state: EspToolState {
                bootloader_path: "".into(),
                firmware_path: "".into(),
                partition_table_path: "".into(),
                archive_path: "".into(),
                storage_path: "".into(),
                ota_data_initial_path: "".into(),
                unpacked_dir: "".into(),
                firmware_elf: "".into(),
                selected_port: "".into(),
            },
        };
    }

    pub fn get_current_esptool(&self) -> String {
        let curr_os = std::env::consts::OS.to_string();
        let curr_arch = std::env::consts::ARCH.to_string();

        return format!("esptool-{}-{}", curr_os, curr_arch);
    }

    pub fn add_file_into_scope(&mut self, file_type: String, filename: String) -> bool {
        if file_type == constants::ESP_FILE_TYPE_BOOTLOADER {
            self.state.bootloader_path = filename;
        } else if file_type == constants::ESP_FILE_TYPE_PARTITION_TABLE {
            self.state.partition_table_path = filename;
        } else if file_type == constants::ESP_FILE_TYPE_FIRMWARE {
            self.state.firmware_path = filename;
        } else if file_type == constants::ESP_FILE_TYPE_ARCHIVE {
            self.state.archive_path = filename;
        } else if file_type == constants::ESP_FILE_TYPE_STORAGE {
            self.state.storage_path = filename;
        } else if file_type == constants::ESP_FILE_TYPE_OTA_DATA_INITIAL {
            self.state.ota_data_initial_path = filename;
        } else if file_type == constants::ESP_FILE_TYPE_UNPACKED_DIR {
            self.state.unpacked_dir = filename;
        } else if file_type == constants::ESP_FILE_FIRMWARE_ELF {
            self.state.firmware_elf = filename;
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

    pub fn initial_create_config_file(&mut self) {
        let path = self.get_config().1;

        if !Config::exists(&path) {
            if Config::write_default(&path) {
                eprintln!("Failed to create config");
            }
        }
    }

    fn validate_is_exists<P: AsRef<Path>>(&mut self, path: P) -> bool {
        path.as_ref().exists()
    }

    pub fn execute_and_listen(&mut self, app: tauri::AppHandle) {
        if self.state.archive_path.is_empty() {
            return;
        }

        let zipper = Zipper::new(self.state.archive_path.clone());
        let _ = zipper.remove_temporary(format!("{}/esptool-gui-temp", self.state.archive_path));
        match zipper.unzip_temporary() {
            Ok(unpacked_dir) => {
                let dir_str = match unpacked_dir.to_str() {
                    Some(s) => s.to_string(),
                    None => {
                        let _ = app.emit("esp-tool-log", "Invalid UTF-8 path");
                        return;
                    }
                };

                self.add_file_into_scope(
                    String::from(constants::ESP_FILE_TYPE_BOOTLOADER),
                    format!("{}/bootloader.bin", dir_str),
                );

                self.add_file_into_scope(
                    String::from(constants::ESP_FILE_TYPE_FIRMWARE),
                    format!("{}/firmware.bin", dir_str),
                );

                self.add_file_into_scope(
                    String::from(constants::ESP_FILE_TYPE_PARTITION_TABLE),
                    format!("{}/partition-table.bin", dir_str),
                );

                let storage_path = format!("{}/storage.bin", dir_str);

                if self.validate_is_exists(&storage_path) {
                    self.add_file_into_scope(
                        String::from(constants::ESP_FILE_TYPE_STORAGE),
                        storage_path,
                    );
                }

                let ota_data_path = format!("{}/ota_data_initial.bin", dir_str);

                if self.validate_is_exists(&ota_data_path) {
                    self.add_file_into_scope(
                        String::from(constants::ESP_FILE_TYPE_OTA_DATA_INITIAL),
                        ota_data_path,
                    );
                }

                self.add_file_into_scope(
                    String::from(constants::ESP_FILE_TYPE_UNPACKED_DIR),
                    format!("{}", dir_str),
                );
            }

            Err(err) => {
                let _ = app.emit("esp-tool-log", err);
            }
        }

        if self.state.bootloader_path.is_empty()
            || self.state.partition_table_path.is_empty()
            || self.state.firmware_path.is_empty()
        {
            return;
        }

        self.stop_flag.store(false, Ordering::Relaxed);
        let stop_flag = self.stop_flag.clone();

        let curr_esptool = self.get_esptool_executor();

        let state = self.state.clone();
        let config = self.get_config().0;

        let handle = std::thread::spawn(move || {
            println!("{}", curr_esptool.display());
            let mut args = Vec::new();

            args.push("--chip".into());
            args.push(config.chip.clone());
            args.push("-b".into());
            args.push(config.baud_rate.to_string());
            args.push("--before".into());

            if !state.selected_port.is_empty() {
                args.push("--port".into());
                args.push(state.selected_port);
            }

            args.extend(config.before_flags.clone());

            args.push("--after".into());
            args.extend(config.after_flags.clone());

            args.extend([
                "--flash_mode".into(),
                config.flash_mode.clone(),
                "--flash_size".into(),
                config.flash_size.clone(),
                "--flash_freq".into(),
                config.flash_freq.clone(),
                config.bootloader_start.clone(),
                state.bootloader_path.clone(),
                config.partition_start.clone(),
                state.partition_table_path.clone(),
                config.firmware_start.clone(),
                state.firmware_path.clone(),
            ]);

            if !state.storage_path.is_empty() {
                args.push(config.storage_start.clone());
                args.push(state.storage_path.clone());
            }

            if !state.ota_data_initial_path.is_empty() {
                args.push(config.ota_initial_data_start.clone());
                args.push(state.ota_data_initial_path.clone());
            }

            let _ = app.emit("esp-tool-log", args.join(" "));

            let mut command = Command::new(curr_esptool)
                .args(args)
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
            let _ = command.wait();

            let dir = state.unpacked_dir.clone();
            if let Err(e) = zipper.remove_temporary(dir.clone()) {
                let _ = app.emit(
                    "esp-tool-log",
                    format!("Failed to cleanup temp dir {}: {}", dir, e),
                );
            } else {
                let _ = app.emit(
                    "esp-tool-log",
                    format!("Temporary directory cleaned: {}", dir),
                );
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

#[tauri::command]
pub fn tauri_monitor_start(app_handle: tauri::AppHandle) {
    let app = ESP_TOOL.get().unwrap().lock().unwrap();

    let mut monitor = ESP_MONITOR.get().unwrap().lock().unwrap();

    monitor.set_baudrate(app.get_config().0.monitor_baud);
    monitor.set_firmware_elf(app.state.firmware_elf.clone());

    monitor.execute_and_listen(app_handle);
}

#[tauri::command]
pub fn tauri_monitor_stop() {
    let mut monitor = ESP_MONITOR.get().unwrap().lock().unwrap();

    monitor.stop_listen();
}

#[tauri::command]
pub fn tauri_get_serial_ports() -> String {
    return get_list_serial_ports();
}


#[tauri::command]
pub fn tauri_set_selected_port(selected: String) {
    let mut app = ESP_TOOL.get().unwrap().lock().unwrap();
    let mut monitor = ESP_MONITOR.get().unwrap().lock().unwrap();

    app.state.selected_port = selected.clone(); 
    monitor.set_port(selected);
}