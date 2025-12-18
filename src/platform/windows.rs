use crate::device::{DeviceType, HealthStatus, SmartData, StorageDevice};
use anyhow::{Context, Result};

#[cfg(target_os = "windows")]
use windows::core::PWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE};
#[cfg(target_os = "windows")]
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Ioctl::{
    IOCTL_DISK_GET_DRIVE_GEOMETRY_EX, IOCTL_STORAGE_QUERY_PROPERTY,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::SystemServices::{GENERIC_READ, GENERIC_WRITE};

pub fn detect_devices() -> Result<Vec<StorageDevice>> {
    #[cfg(target_os = "windows")]
    {
        detect_devices_impl()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}

#[cfg(target_os = "windows")]
fn detect_devices_impl() -> Result<Vec<StorageDevice>> {
    use widestring::U16CString;
    use windows::Win32::System::Ioctl::{DISK_GEOMETRY_EX, STORAGE_DEVICE_NUMBER};

    let mut devices = Vec::new();

    // Try to open physical drives 0-15
    for i in 0..16 {
        let device_path = format!("\\\\.\\PhysicalDrive{}", i);

        let wide_path = match U16CString::from_str(&device_path) {
            Ok(path) => path,
            Err(_) => continue,
        };

        unsafe {
            let handle = CreateFileW(
                PWSTR(wide_path.as_ptr() as *mut u16),
                GENERIC_READ.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                Default::default(),
                None,
            );

            if handle.is_err() || handle.as_ref().unwrap().is_invalid() {
                continue;
            }

            let handle = handle.unwrap();

            // Get disk geometry to determine size
            let capacity = get_disk_capacity(handle);

            if capacity == 0 {
                let _ = windows::Win32::Foundation::CloseHandle(handle);
                continue;
            }

            // Try to get device descriptor for model and serial
            let (model, serial) = get_device_descriptor(handle);

            let _ = windows::Win32::Foundation::CloseHandle(handle);

            devices.push(StorageDevice {
                path: device_path,
                model: model.unwrap_or_else(|| format!("Physical Drive {}", i)),
                serial: serial.unwrap_or_else(|| "Unknown".to_string()),
                capacity,
                device_type: DeviceType::Unknown,
                is_removable: false,
                interface: "Unknown".to_string(),
            });
        }
    }

    Ok(devices)
}

#[cfg(target_os = "windows")]
unsafe fn get_disk_capacity(handle: HANDLE) -> u64 {
    use std::mem;
    use windows::Win32::System::Ioctl::DISK_GEOMETRY_EX;
    use windows::Win32::System::IO::DeviceIoControl;

    let mut geometry: DISK_GEOMETRY_EX = mem::zeroed();
    let mut bytes_returned: u32 = 0;

    let result = DeviceIoControl(
        handle,
        IOCTL_DISK_GET_DRIVE_GEOMETRY_EX,
        None,
        0,
        Some(&mut geometry as *mut _ as *mut _),
        mem::size_of::<DISK_GEOMETRY_EX>() as u32,
        Some(&mut bytes_returned),
        None,
    );

    if result.is_ok() {
        geometry.DiskSize as u64
    } else {
        0
    }
}

#[cfg(target_os = "windows")]
unsafe fn get_device_descriptor(handle: HANDLE) -> (Option<String>, Option<String>) {
    use std::mem;
    use windows::Win32::System::IO::DeviceIoControl;

    // This is a simplified version - a full implementation would use
    // STORAGE_PROPERTY_QUERY and STORAGE_DEVICE_DESCRIPTOR
    // For now, return None
    (None, None)
}

pub fn has_admin_privileges() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Check if running as administrator
        // This is a simplified check
        use std::process::Command;

        let output = Command::new("net").args(&["session"]).output();

        match output {
            Ok(out) => out.status.success(),
            Err(_) => false,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn read_smart_data(_device: &StorageDevice) -> Result<SmartData> {
    // Placeholder implementation
    Ok(SmartData {
        attributes: Vec::new(),
        health_status: HealthStatus::Unknown,
        temperature: None,
        power_on_hours: None,
    })
}

pub fn is_device_mounted(_device: &StorageDevice) -> Result<bool> {
    // On Windows, check if any volume is using this physical drive
    // This is a simplified implementation
    Ok(false)
}
