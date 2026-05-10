use crate::device::{DeviceType, StorageDevice};
use anyhow::{Context, Result};
use std::path::Path;

const MAX_DISKS: u32 = 64;

pub fn detect_devices() -> Result<Vec<StorageDevice>> {
    let mut devices = Vec::new();

    for i in 0..MAX_DISKS {
        let device_path = format!("/dev/disk{}", i);

        if !Path::new(&device_path).exists() {
            continue;
        }

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

    let output = Command::new("diskutil")
        .args(["info", "-plist", device_path])
        .output()
        .context("Failed to execute diskutil")?;

    if !output.status.success() {
        anyhow::bail!("diskutil command failed");
    }

    let plist_value: plist::Value =
        plist::from_bytes(&output.stdout).context("Failed to parse diskutil plist output")?;

    let dict = match plist_value.as_dictionary() {
        Some(d) => d,
        None => anyhow::bail!("plist output is not a dictionary"),
    };

    let model = dict
        .get("MediaName")
        .and_then(|v| v.as_string())
        .or_else(|| dict.get("DeviceIdentifier").and_then(|v| v.as_string()))
        .unwrap_or("Unknown")
        .to_string();

    let serial = dict
        .get("SerialNumber")
        .and_then(|v| v.as_string())
        .unwrap_or("Unknown")
        .to_string();

    let capacity: u64 = dict
        .get("TotalSize")
        .and_then(|v| v.as_signed_integer())
        .map(|v| v as u64)
        .unwrap_or(0);

    let is_removable = dict
        .get("Removable")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false);

    let device_type = if dict
        .get("SolidState")
        .and_then(|v| v.as_boolean())
        .unwrap_or(false)
    {
        DeviceType::SSD
    } else if dict
        .get("Protocol")
        .and_then(|v| v.as_string())
        .map_or(false, |s| s.contains("USB"))
    {
        DeviceType::USB
    } else {
        DeviceType::HDD
    };

    let interface = dict
        .get("Protocol")
        .and_then(|v| v.as_string())
        .unwrap_or("Unknown")
        .to_string();

    Ok(StorageDevice {
        path: device_path.to_string(),
        model,
        serial,
        capacity,
        device_type,
        is_removable,
        interface,
    })
}

pub fn has_admin_privileges() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub fn is_device_mounted(device: &StorageDevice) -> Result<bool> {
    use std::process::Command;

    let output = Command::new("diskutil")
        .args(["info", &device.path])
        .output()
        .context("Failed to execute diskutil")?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    Ok(output_str.contains("Mounted: Yes") || output_str.contains("Volume Name:"))
}

pub fn open_device_exclusive(path: &str) -> Result<std::fs::File> {
    use std::os::unix::io::AsRawFd;

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .context("Failed to open device")?;

    // Acquire exclusive lock
    let ret = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if ret != 0 {
        let err = std::io::Error::last_os_error();
        anyhow::bail!("Failed to acquire exclusive lock on {}: {}", path, err);
    }

    Ok(file)
}
