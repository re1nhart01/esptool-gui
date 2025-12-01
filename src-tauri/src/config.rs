use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct About {
    pub name: String,
    pub version: String,
    pub author: String,
    pub date_of_release: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub chip: String,
    pub baud_rate: u32,
    pub before_flags: Vec<String>,
    pub after_flags: Vec<String>,
    pub flash_mode: String,
    pub flash_size: String,
    pub flash_freq: String,
    pub bootloader_start: String,
    pub partition_start: String,
    pub firmware_start: String,
    pub about: About,
}

impl Config {
    pub fn new() -> Self {
        return Self {
            about: About {
                author: String::from("Eugene Kokaiko"),
                date_of_release: String::from("01.01.1970"),
                name: String::from("ESP_FLASH-GUI"),
                version: String::from("0.0.1"),
            },
            after_flags: vec![String::from("hard_reset"), String::from("write_flash")],
            before_flags: vec![String::from("default_reset")],
            baud_rate: 460800,
            bootloader_start: String::from("0x0"),
            partition_start: String::from("0x8000"),
            chip: String::from("esp32s3"),
            firmware_start: String::from("0x10000"),
            flash_freq: String::from("80m"),
            flash_mode: String::from("dio"),
            flash_size: String::from("8MB"),
        };
    }
    pub fn get(&self) -> &Self {
        return self;
    }

    pub fn update_config(&mut self, new_cfg: Config, path: &Path) -> bool {
        *self = new_cfg;

        match serde_json::to_string_pretty(self) {
            Ok(cfg_json) => {
                if fs::write(path, cfg_json).is_ok() {
                    return true;
                }
            }
            Err(_) => {}
        }
        false
    }
}
