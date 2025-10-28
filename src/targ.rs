#![allow(dead_code)]
#![allow(unused_imports)]

use std::fs;
use std::io::{self, Result, Write, stdout};
use std::path::PathBuf;
use std::process::Command;

use crossterm::{
    execute,
    cursor,
    style::Stylize,
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
    event::{self, KeyCode, Event},
};

/// Unified structure for displaying drives
#[derive(Debug, Clone)]
struct DriveInfo {
    path: String,
    model: Option<String>,
}

/// Windows: list removable drives with model names
#[cfg(target_os = "windows")]
fn list_flashable_drives_windows() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    let output = Command::new("powershell")
        .args([
            "-Command",
            "Get-CimInstance Win32_DiskDrive | Where-Object { $_.MediaType -eq 'Removable Media' -or $_.InterfaceType -eq 'USB' } | Select-Object DeviceID, Model",
        ])
        .output()
        .expect("failed to run PowerShell command");

    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.trim().is_empty() || line.contains("DeviceID") {
            continue;
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let device_id = parts[0].trim().to_string();
            let model = parts[1..].join(" ");
            drives.push(DriveInfo { path: device_id, model: Some(model) });
        } else if parts.len() == 1 && parts[0].starts_with("\\\\.\\") {
            drives.push(DriveInfo { path: parts[0].trim().to_string(), model: None });
        }
    }

    drives
}

/// macOS: list external drives with model names
#[cfg(target_os = "macos")]
fn list_flashable_drives_macos() -> Vec<DriveInfo> {
    let mut drives = Vec::new();

    let output = Command::new("diskutil").arg("list").output().expect("failed to run diskutil");
    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if line.contains("external, physical") {
            if let Some(disk_name) = line.split_whitespace().next() {
                let path = format!("/dev/{}", disk_name);
                let info_output = Command::new("diskutil").args(["info", &path]).output().unwrap();
                let info_text = String::from_utf8_lossy(&info_output.stdout);

                let mut model = None;
                for infoline in info_text.lines() {
                    if infoline.contains("Device / Media Name:") {
                        model = Some(infoline.split(':').nth(1).unwrap_or("").trim().to_string());
                        break;
                    }
                }

                drives.push(DriveInfo { path, model });
            }
        }
    }

    drives
}

/// Linux: list removable drives with model names
#[cfg(target_os = "linux")]
fn list_flashable_drives_linux() -> Result<Vec<DriveInfo>> {
    let mut drives = Vec::new();

    for entry in fs::read_dir("/sys/block")? {
        let entry = entry?;
        let dev_str_os = entry.file_name();
        let dev_str = dev_str_os.to_string_lossy().to_string(); // convert to owned String
        let removable_path = format!("/sys/block/{}/removable", dev_str);

        if let Ok(contents) = fs::read_to_string(&removable_path) {
            if contents.trim() == "1" {
                let model_path = format!("/sys/block/{}/device/model", dev_str);
                let model = fs::read_to_string(&model_path).ok().map(|s| s.trim().to_string());
                let dev_path = format!("/dev/{}", dev_str);
                if fs::metadata(&dev_path).is_ok() {
                    drives.push(DriveInfo { path: dev_path, model });
                }
            }
        }
    }

    Ok(drives)
}

/// Menu UI for selecting which drive to flash to
pub fn menu() -> Result<Option<String>> {
    let mut stdout = stdout();
    print!("\x1B[H\x1B[2J");
    io::stdout().flush()?;

    enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    let extdevs: Vec<DriveInfo> = {
        #[cfg(target_os = "windows")] { list_flashable_drives_windows() }
        #[cfg(target_os = "macos")] { list_flashable_drives_macos() }
        #[cfg(target_os = "linux")] { list_flashable_drives_linux()? }
    };

    if extdevs.is_empty() {
        println!("No removable drives detected.");
        println!("Insert a USB drive and restart the program.");
        disable_raw_mode()?;
        return Ok(None);
    }

    let mut extselected = 0;

    loop {
        execute!(stdout, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        println!("External devices found:");

        for (i, item) in extdevs.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, (i + 1) as u16))?;
            execute!(stdout, terminal::Clear(ClearType::CurrentLine))?;

            let label = if let Some(model) = &item.model {
                format!("{} — {}", item.path, model)
            } else {
                item.path.clone()
            };

            if i == extselected {
                println!("{}", label.on_white().black());
            } else {
                println!("{}", label);
            }
        }

        stdout.flush()?;

        if let Event::Key(ev) = event::read()? {
            match ev.code {
                KeyCode::Up => { if extselected > 0 { extselected -= 1; } }
                KeyCode::Down => { if extselected < extdevs.len() - 1 { extselected += 1; } }
                KeyCode::Enter => {
                    let selected_device = extdevs[extselected].clone();
                    disable_raw_mode()?;
                    execute!(stdout, cursor::Show)?;
                    return Ok(Some(selected_device.path));
                }         
                KeyCode::Esc => { disable_raw_mode()?; execute!(stdout, cursor::Show)?; return Ok(None); }
                _ => {}
            }
        }
    }
}
