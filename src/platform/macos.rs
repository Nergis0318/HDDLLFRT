use crate::device::{DeviceType, HealthStatus, SmartData, StorageDevice};
use anyhow::{Context, Result};
use std::path::Path;

pub fn detect_devices() -> Result<Vec<StorageDevice>> {
    let mut devices = Vec::new();

    // On macOS, disk devices are typically /dev/diskN
    for i in 0..20 {
        let device_path = format!("/dev/disk{}", i);

        if !Path::new(&device_path).exists() {
            continue;
        }

        // Try to get device information using diskutil
        if let Ok(info) = get_diskutil_info(&device_path) {
            if info.capacity > 0 {
                devices.push(info);
            }
        }
    }

    Ok(devices)
}

fn get_diskutil_info(device_path: &str) -> Result<StorageDevice> {
    use std::process::Command;

    // Use diskutil to get device information
    let output = Command::new("diskutil")
        .args(&["info", "-plist", device_path])
        .output()
        .context("Failed to execute diskutil")?;

    if !output.status.success() {
        anyhow::bail!("diskutil command failed");
    }

    // Parse the plist output (simplified - real implementation would use plist parser)
    let output_str = String::from_utf8_lossy(&output.stdout);

    // Extract basic information (this is a simplified parser)
    let model = extract_plist_value(&output_str, "MediaName")
        .or_else(|| extract_plist_value(&output_str, "DeviceIdentifier"))
        .unwrap_or_else(|| "Unknown".to_string());

    let size_str = extract_plist_value(&output_str, "TotalSize").unwrap_or_else(|| "0".to_string());
    let capacity: u64 = size_str.parse().unwrap_or(0);

    // Determine if removable
    let removable_str =
        extract_plist_value(&output_str, "Removable").unwrap_or_else(|| "false".to_string());
    let is_removable = removable_str.contains("true");

    // Determine device type
    let device_type = if output_str.contains("SSD") || output_str.contains("Solid State") {
        DeviceType::SSD
    } else if output_str.contains("USB") {
        DeviceType::USB
    } else {
        DeviceType::HDD
    };

    let interface = if output_str.contains("USB") {
        "USB".to_string()
    } else if output_str.contains("SATA") {
        "SATA".to_string()
    } else {
        "Unknown".to_string()
    };

    Ok(StorageDevice {
        path: device_path.to_string(),
        model,
        serial: "Unknown".to_string(),
        capacity,
        device_type,
        is_removable,
        interface,
    })
}

fn extract_plist_value(plist: &str, key: &str) -> Option<String> {
    // Very simple plist value extraction
    let key_line = format!("<key>{}</key>", key);

    if let Some(pos) = plist.find(&key_line) {
        let after_key = &plist[pos + key_line.len()..];

        // Look for the value in different formats
        if let Some(string_start) = after_key.find("<string>") {
            if let Some(string_end) = after_key[string_start..].find("</string>") {
                let value = &after_key[string_start + 8..string_start + string_end];
                return Some(value.trim().to_string());
            }
        }

        if let Some(integer_start) = after_key.find("<integer>") {
            if let Some(integer_end) = after_key[integer_start..].find("</integer>") {
                let value = &after_key[integer_start + 9..integer_start + integer_end];
                return Some(value.trim().to_string());
            }
        }

        if after_key.trim_start().starts_with("<true/>") {
            return Some("true".to_string());
        }

        if after_key.trim_start().starts_with("<false/>") {
            return Some("false".to_string());
        }
    }

    None
}

pub fn has_admin_privileges() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub fn read_smart_data(_device: &StorageDevice) -> Result<SmartData> {
    // Could use smartctl on macOS, but for now return placeholder
    Ok(SmartData {
        attributes: Vec::new(),
        health_status: HealthStatus::Unknown,
        temperature: None,
        power_on_hours: None,
    })
}

pub fn is_device_mounted(device: &StorageDevice) -> Result<bool> {
    use std::process::Command;

    // Use diskutil to check if mounted
    let output = Command::new("diskutil")
        .args(&["info", &device.path])
        .output()
        .context("Failed to execute diskutil")?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.contains("Mounted: Yes") || output_str.contains("Volume Name:"))
}

pub fn open_device_exclusive(path: &str) -> Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .context("Failed to open device")
}
