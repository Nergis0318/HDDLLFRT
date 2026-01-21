use crate::device::{DeviceType, StorageDevice};
use anyhow::{Context, Result};
use std::fs;
use std::os::unix::io::AsRawFd;
use std::path::Path;

// NVMe ioctl definition
#[repr(C)]
#[derive(Debug, Default)]
struct NvmeAdminCmd {
    opcode: u8,
    flags: u8,
    rsvd1: u16,
    nsid: u32,
    cdw2: u32,
    cdw3: u32,
    metadata: u64,
    addr: u64,
    metadata_len: u32,
    data_len: u32,
    cdw10: u32,
    cdw11: u32,
    cdw12: u32,
    cdw13: u32,
    cdw14: u32,
    cdw15: u32,
    timeout_ms: u32,
    result: u32,
}

const NVME_IOCTL_ID: u8 = b'N';
const NVME_IOCTL_ADMIN_CMD_SEQ: u8 = 0x41;

nix::ioctl_readwrite!(
    nvme_admin_cmd,
    NVME_IOCTL_ID,
    NVME_IOCTL_ADMIN_CMD_SEQ,
    NvmeAdminCmd
);

/// Detect all block devices on Linux
pub fn detect_devices() -> Result<Vec<StorageDevice>> {
    let mut devices = Vec::new();
    let sys_block = Path::new("/sys/block");

    if !sys_block.exists() {
        return Ok(devices);
    }

    for entry in fs::read_dir(sys_block)? {
        let entry = entry?;
        let device_name = entry.file_name();
        let device_name_str = device_name.to_string_lossy();

        // Skip loop devices, ram devices, etc.
        if device_name_str.starts_with("loop")
            || device_name_str.starts_with("ram")
            || device_name_str.starts_with("dm-")
        {
            continue;
        }

        let device_path = format!("/dev/{}", device_name_str);

        // Check if device exists and is a block device
        if !Path::new(&device_path).exists() {
            continue;
        }

        // Read device information
        let sys_path = entry.path();

        let model = read_sys_file(&sys_path.join("device/model"))
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string();

        let serial = read_sys_file(&sys_path.join("device/serial"))
            .unwrap_or_else(|_| "Unknown".to_string())
            .trim()
            .to_string();

        // Read capacity (in 512-byte sectors)
        let size_str = read_sys_file(&sys_path.join("size"))?;
        let sectors: u64 = size_str.trim().parse().unwrap_or(0);
        let capacity = sectors * 512;

        // Skip devices with zero capacity
        if capacity == 0 {
            continue;
        }

        // Determine device type
        let device_type = determine_device_type(&sys_path);

        // Check if removable
        let removable_str =
            read_sys_file(&sys_path.join("removable")).unwrap_or_else(|_| "0".to_string());
        let is_removable = removable_str.trim() == "1";

        // Determine interface type
        let interface = determine_interface(&sys_path);

        devices.push(StorageDevice {
            path: device_path,
            model,
            serial,
            capacity,
            device_type,
            is_removable,
            interface,
        });
    }

    Ok(devices)
}

fn read_sys_file(path: &Path) -> Result<String> {
    fs::read_to_string(path).context(format!("Failed to read {}", path.display()))
}

fn determine_device_type(sys_path: &Path) -> DeviceType {
    // Check if it's an NVMe device
    if sys_path.to_string_lossy().contains("nvme") {
        return DeviceType::NVMe;
    }

    // Check rotation rate to determine if it's SSD or HDD
    if let Ok(rotation) = read_sys_file(&sys_path.join("queue/rotational")) {
        if rotation.trim() == "0" {
            return DeviceType::SSD;
        } else {
            return DeviceType::HDD;
        }
    }

    // Check if it's a USB device
    if let Ok(_) = read_sys_file(&sys_path.join("device/../../idVendor")) {
        return DeviceType::USB;
    }

    DeviceType::Unknown
}

fn determine_interface(sys_path: &Path) -> String {
    // Check for NVMe
    if sys_path.to_string_lossy().contains("nvme") {
        return "NVMe".to_string();
    }

    // Check for USB
    if read_sys_file(&sys_path.join("device/../../idVendor")).is_ok() {
        return "USB".to_string();
    }

    // Check for ATA/SATA
    if read_sys_file(&sys_path.join("device/vendor")).is_ok() {
        return "SATA".to_string();
    }

    "Unknown".to_string()
}

pub fn has_admin_privileges() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub fn is_device_mounted(device: &StorageDevice) -> Result<bool> {
    // Read /proc/mounts to check if device or its partitions are mounted
    let mounts = fs::read_to_string("/proc/mounts")?;

    // Check if the device itself is mounted
    if mounts.contains(&device.path) {
        return Ok(true);
    }

    // Check if any partition is mounted (e.g., /dev/sda1, /dev/sda2)
    let device_base = device.path.trim_end_matches(char::is_numeric);
    for line in mounts.lines() {
        if let Some(mount_device) = line.split_whitespace().next() {
            if mount_device.starts_with(device_base) {
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// Open a device for exclusive access
pub fn open_device_exclusive(path: &str) -> Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .context("Failed to open device")
}
