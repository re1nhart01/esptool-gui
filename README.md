# ESP32S3 Firmware Flash Tool (with GUI)

A cross‑platform GUI application built with **Tauri**, **Rust**, and **React (shadcn/ui)** to simplify flashing firmware onto **ESP32‑S3** devices.

This tool allows you to:
- Select Bootloader / Partition Table / Firmware binaries
- Configure flashing parameters (chip model, baud rate, flash mode, etc.)
- Update and persist configuration settings in JSON
- Flash ESP via a single command button
- View detailed flashing logs in real‑time

---

## 🚀 Features

| Feature | Status |
|--------|--------|
| Select binaries via GUI | ✅ |
| Flash config stored in JSON | ✅ |
| Automatic settings reload | ✅ |
| Flash log stream in UI | ✅ |
| Using esptool under the hood | ✅ |
| UI based on shadcn/ui | ✅ |
| Tauri + Rust backend | ✅ |

---

## 🛠 Tech Stack

| Layer | Technology |
|------|------------|
| GUI | React + shadcn/ui |
| Backend | Rust + Tauri commands |
| Flashing engine | esptool.py (embedded binary) |
| Config format | Serde JSON |

---

## 📂 File Selection

The **Flash** tab allows selecting:

- Bootloader binary (`bootloader.bin`)
- Partition table (`partition-table.bin`)
- Firmware (`.bin`)

These values are persisted and restored whenever the app is reopened.

---

## ⚙ Settings

The **Settings** tab exposes configuration options directly from Rust `Config` struct:

```rust
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
