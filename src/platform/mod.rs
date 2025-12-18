use crate::device::{SmartData, StorageDevice};
use anyhow::Result;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// Detect all storage devices on the system
pub fn detect_devices() -> Result<Vec<StorageDevice>> {
    #[cfg(target_os = "linux")]
    return linux::detect_devices();

    #[cfg(target_os = "windows")]
    return windows::detect_devices();

    #[cfg(target_os = "macos")]
    return macos::detect_devices();

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        anyhow::bail!("Unsupported operating system");
    }
}

/// Check if the current process has administrative privileges
pub fn has_admin_privileges() -> bool {
    #[cfg(target_os = "linux")]
    return linux::has_admin_privileges();

    #[cfg(target_os = "windows")]
    return windows::has_admin_privileges();

    #[cfg(target_os = "macos")]
    return macos::has_admin_privileges();

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    return false;
}

/// Read S.M.A.R.T. data from a device
pub fn read_smart_data(device: &StorageDevice) -> Result<SmartData> {
    #[cfg(target_os = "linux")]
    return linux::read_smart_data(device);

    #[cfg(target_os = "windows")]
    return windows::read_smart_data(device);

    #[cfg(target_os = "macos")]
    return macos::read_smart_data(device);

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        anyhow::bail!("Unsupported operating system");
    }
}

/// Check if a device is mounted
pub fn is_device_mounted(device: &StorageDevice) -> Result<bool> {
    #[cfg(target_os = "linux")]
    return linux::is_device_mounted(device);

    #[cfg(target_os = "windows")]
    return windows::is_device_mounted(device);

    #[cfg(target_os = "macos")]
    return macos::is_device_mounted(device);

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    return Ok(false);
}
