use crate::device::StorageDevice;
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

use std::io::{self, Read, Seek, Write};

/// A wrapper around a file handle that may hold additional resources (like locks)
pub struct DeviceHandle {
    file: std::fs::File,
    #[cfg(target_os = "windows")]
    _locks: Vec<std::fs::File>,
}

impl Read for DeviceHandle {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for DeviceHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl DeviceHandle {
    pub fn sync_all(&self) -> io::Result<()> {
        self.file.sync_all()
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        let _ = self.file.sync_all();
    }
}

impl DeviceHandle {
    #[cfg(unix)]
    pub fn write_at(&self, buf: &[u8], offset: u64) -> io::Result<usize> {
        use std::os::unix::fs::FileExt;
        self.file.write_at(buf, offset)
    }

    #[cfg(unix)]
    pub fn read_at(&self, buf: &mut [u8], offset: u64) -> io::Result<usize> {
        use std::os::unix::fs::FileExt;
        self.file.read_at(buf, offset)
    }
}

impl Seek for DeviceHandle {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

/// Open a device for exclusive access (write)
pub fn open_device_exclusive(path: &str) -> Result<DeviceHandle> {
    #[cfg(target_os = "linux")]
    return linux::open_device_exclusive(path).map(|f| DeviceHandle { file: f });

    #[cfg(target_os = "windows")]
    return windows::open_device_exclusive(path);

    #[cfg(target_os = "macos")]
    return macos::open_device_exclusive(path).map(|f| DeviceHandle { file: f });

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
