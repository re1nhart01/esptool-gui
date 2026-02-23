use std::{io::{Read}, path::PathBuf, sync::{Arc, atomic::{AtomicBool, Ordering}}, thread::JoinHandle};

use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use tauri::Emitter;


pub struct Monitor {
    thread_handle: Option<JoinHandle<()>>,
    stop_flag: Arc<AtomicBool>,
    firmware_elf: String,
    baudrate: String,
    port: String,
}

impl Monitor {
    pub fn new() -> Self {
        return Self {
            thread_handle: None,
            firmware_elf: String::from(""),
            baudrate: String::from("115200"),
            port: String::from(""),
            stop_flag: Arc::new(AtomicBool::new(true)),
        };
    }

    pub fn set_firmware_elf(&mut self, elf: String) {
        self.firmware_elf = elf;
    }

    pub fn set_baudrate(&mut self, baud: String) {
        self.baudrate = baud;
    }

    pub fn set_port(&mut self, port: String) {
        self.port = port;
    }

    pub fn get_current_espmonitor(&self) -> String {
        let curr_os = std::env::consts::OS.to_string();
        let curr_arch = std::env::consts::ARCH.to_string();

        return format!("idf-monitor-{}-{}", curr_os, curr_arch);
    }

    fn get_espmonitor_executor(&self) -> PathBuf {
        let exe = std::env::current_exe().unwrap();
        let cwd = exe.parent().unwrap();

        return cwd
            .join("vendor")
            .join(self.get_current_espmonitor())
            .join(if cfg!(windows) {
                "esp_idf_monitor.exe"
            } else {
                "esp_idf_monitor"
            });
    }

pub fn execute_and_listen(&mut self, app: tauri::AppHandle) {
    self.stop_flag.store(false, Ordering::Relaxed);

    let monitor = self.get_espmonitor_executor();
    let stop_flag = self.stop_flag.clone();

    let port = self.port.clone();
    let baud = self.baudrate.clone();
    let elf  = self.firmware_elf.clone();

    let handle = std::thread::spawn(move || {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize { rows: 24, cols: 80, pixel_width: 0, pixel_height: 0 })
            .expect("Failed to open PTY");

        let mut cmd = CommandBuilder::new(monitor);

        if !port.is_empty() {
            cmd.arg("--port");
            cmd.arg(port);
        }

        if !baud.is_empty() {
            cmd.arg("--baud");
            cmd.arg(baud);
        }
        if !elf.is_empty() {
            cmd.arg(elf);
        }

        let mut child = match pair.slave.spawn_command(cmd) {
            Ok(c) => c,
            Err(e) => {
                let _ = app.emit("esp-tool-monitor", format!("spawn error: {e}"));
                return;
            }
        };

        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader().expect("Failed to get reader");
        let mut buffer = [0u8; 4096];

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                let _ = child.kill();
                break;
            }

            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                    let _ = app.emit("esp-tool-monitor", output);
                }
                Err(e) => {
                    let _ = app.emit("esp-tool-monitor", format!("pty read error: {e}"));
                    break;
                }
            }
        }

        let _ = child.wait();
    });

    self.thread_handle = Some(handle);
}

    pub fn stop_listen(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            handle.join().unwrap();
        }
    }
}