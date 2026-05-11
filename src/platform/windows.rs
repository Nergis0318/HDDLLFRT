use crate::device::{DeviceType, StorageDevice};
use crate::platform::DeviceHandle;
use anyhow::{Context, Result};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HANDLE;
#[cfg(target_os = "windows")]
use windows::Win32::Storage::FileSystem::{
    CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, GetLogicalDrives, OPEN_EXISTING,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Ioctl::IOCTL_DISK_GET_DRIVE_GEOMETRY_EX;
#[cfg(target_os = "windows")]
use windows::Win32::System::Ioctl::{
    DEVICE_SEEK_PENALTY_DESCRIPTOR, FSCTL_DISMOUNT_VOLUME, FSCTL_LOCK_VOLUME,
    IOCTL_STORAGE_GET_DEVICE_NUMBER, IOCTL_STORAGE_QUERY_PROPERTY, PropertyStandardQuery,
    STORAGE_DEVICE_DESCRIPTOR, STORAGE_DEVICE_NUMBER, STORAGE_PROPERTY_QUERY,
    StorageDeviceProperty, StorageDeviceSeekPenaltyProperty,
};
#[cfg(target_os = "windows")]
use windows::core::PCWSTR;

const GENERIC_READ: u32 = 0x80000000;

const MAX_PHYSICAL_DRIVES: u32 = 64;

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

    let mut devices = Vec::new();

    for i in 0..MAX_PHYSICAL_DRIVES {
        let device_path = format!("\\\\.\\PhysicalDrive{}", i);

        let wide_path = match U16CString::from_str(&device_path) {
            Ok(path) => path,
            Err(_) => continue,
        };

        unsafe {
            let handle = CreateFileW(
                PCWSTR(wide_path.as_ptr()),
                GENERIC_READ,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                Default::default(),
                None,
            );

            let handle: HANDLE = match handle {
                Ok(h) if !h.is_invalid() => h,
                _ => continue,
            };

            // Get disk geometry to determine size
            let capacity = get_disk_capacity(handle);

            if capacity == 0 {
                let _ = windows::Win32::Foundation::CloseHandle(handle);
                continue;
            }

            // Try to get device details
            let details = get_device_details(handle);

            let _ = windows::Win32::Foundation::CloseHandle(handle);

            devices.push(StorageDevice {
                path: device_path,
                model: details
                    .model
                    .unwrap_or_else(|| format!("Physical Drive {}", i)),
                serial: details.serial.unwrap_or_else(|| "Unknown".to_string()),
                capacity,
                device_type: details.device_type,
                is_removable: details.is_removable,
                interface: details.interface,
            });
        }
    }

    Ok(devices)
}

#[cfg(target_os = "windows")]
unsafe fn get_disk_capacity(handle: HANDLE) -> u64 {
    unsafe {
        use std::mem;
        use windows::Win32::System::IO::DeviceIoControl;
        use windows::Win32::System::Ioctl::DISK_GEOMETRY_EX;

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
}

#[cfg(target_os = "windows")]
struct DeviceDetails {
    model: Option<String>,
    serial: Option<String>,
    interface: String,
    device_type: DeviceType,
    is_removable: bool,
}

#[cfg(target_os = "windows")]
unsafe fn get_device_details(handle: HANDLE) -> DeviceDetails {
    unsafe {
        use std::ffi::CStr;
        use std::mem;
        use windows::Win32::System::IO::DeviceIoControl;

        let mut query = STORAGE_PROPERTY_QUERY {
            PropertyId: StorageDeviceProperty,
            QueryType: PropertyStandardQuery,
            AdditionalParameters: [0],
        };

        let mut buffer = [0u8; 1024];
        let mut bytes_returned = 0;

        let result = DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            Some(&mut query as *mut _ as *mut _),
            mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            buffer.len() as u32,
            Some(&mut bytes_returned),
            None,
        );

        let mut model = None;
        let mut serial = None;
        let mut interface = "Unknown".to_string();
        let mut is_removable = false;
        let mut device_type = DeviceType::Unknown;

        if result.is_ok() {
            let descriptor = &*(buffer.as_ptr() as *const STORAGE_DEVICE_DESCRIPTOR);

            // Parse strings
            if descriptor.ProductIdOffset > 0 {
                let ptr = buffer.as_ptr().add(descriptor.ProductIdOffset as usize) as *const i8;
                if let Ok(s) = CStr::from_ptr(ptr).to_str() {
                    model = Some(s.trim().to_string());
                }
            }

            if descriptor.SerialNumberOffset > 0 {
                let ptr = buffer.as_ptr().add(descriptor.SerialNumberOffset as usize) as *const i8;
                if let Ok(s) = CStr::from_ptr(ptr).to_str() {
                    serial = Some(s.trim().to_string());
                }
            }

            is_removable = descriptor.RemovableMedia.as_bool();

            interface = match descriptor.BusType.0 {
                0x7 => "USB".to_string(),      // BusTypeUsb
                0xB => "SATA".to_string(),     // BusTypeSata
                0x11 => "NVMe".to_string(),    // BusTypeNvme
                0x2 => "IDE".to_string(),      // BusTypeAtapi
                0x3 => "ATA".to_string(),      // BusTypeAta
                0x1 => "SCSI".to_string(),     // BusTypeScsi
                0x8 => "RAID".to_string(),     // BusTypeRAID
                0x4 => "FireWire".to_string(), // BusType1394
                0x5 => "SSA".to_string(),      // BusTypeSsa
                0x6 => "Fibre".to_string(),    // BusTypeFibre
                0xA => "SAS".to_string(),      // BusTypeSas
                _ => format!("Unknown ({})", descriptor.BusType.0),
            };

            if descriptor.BusType.0 == 0x11 {
                // BusTypeNvme
                device_type = DeviceType::NVMe;
            } else if descriptor.BusType.0 == 0x7 {
                // BusTypeUsb
                device_type = DeviceType::USB;
            }
        }

        // Check for seek penalty to distinguish HDD/SSD
        if device_type == DeviceType::Unknown || device_type == DeviceType::USB {
            let mut query_seek = STORAGE_PROPERTY_QUERY {
                PropertyId: StorageDeviceSeekPenaltyProperty,
                QueryType: PropertyStandardQuery,
                AdditionalParameters: [0],
            };
            let mut seek_buffer = [0u8; 1024];
            let mut seek_bytes = 0;

            let seek_result = DeviceIoControl(
                handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                Some(&mut query_seek as *mut _ as *mut _),
                mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                Some(seek_buffer.as_mut_ptr() as *mut _),
                seek_buffer.len() as u32,
                Some(&mut seek_bytes),
                None,
            );

            if seek_result.is_ok() {
                let descriptor = &*(seek_buffer.as_ptr() as *const DEVICE_SEEK_PENALTY_DESCRIPTOR);
                if !descriptor.IncursSeekPenalty.as_bool() {
                    if device_type == DeviceType::Unknown {
                        device_type = DeviceType::SSD;
                    }
                } else if device_type == DeviceType::Unknown {
                    device_type = DeviceType::HDD;
                }
            }
        }

        DeviceDetails {
            model,
            serial,
            interface,
            device_type,
            is_removable,
        }
    }
}

pub fn has_admin_privileges() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        let output = Command::new("net").args(["session"]).output();
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

pub fn is_device_mounted(device: &StorageDevice) -> Result<bool> {
    #[cfg(target_os = "windows")]
    {
        // Parse drive number from path
        let drive_num = if device.path.starts_with(r"\\.\PhysicalDrive") {
            device
                .path
                .trim_start_matches(r"\\.\PhysicalDrive")
                .parse::<u32>()
                .ok()
        } else {
            return Ok(false);
        };

        let Some(target_disk_num) = drive_num else {
            return Ok(false);
        };

        // Enumerate logical drives and check if any volume belongs to this disk
        let drives = unsafe { GetLogicalDrives() };
        for i in 0..26 {
            if (drives & (1 << i)) != 0 {
                let drive_letter = (b'A' + i as u8) as char;
                let volume_path = format!(r"\\.\{}:", drive_letter);

                if let Ok(volume_file) = std::fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(&volume_path)
                {
                    use std::os::windows::prelude::*;
                    let handle = HANDLE(volume_file.as_raw_handle() as _);
                    if let Ok(disk_num) = get_volume_disk_number(handle) {
                        if disk_num == target_disk_num {
                            return Ok(true);
                        }
                    }
                }
            }
        }

        Ok(false)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = device;
        Ok(false)
    }
}

#[cfg(target_os = "windows")]
pub fn open_device_exclusive(path: &str) -> Result<DeviceHandle> {
    use std::fs::OpenOptions;
    use std::os::windows::prelude::*;

    // Parse drive number from path
    let drive_num = if path.starts_with(r"\\.\PhysicalDrive") {
        path.trim_start_matches(r"\\.\PhysicalDrive")
            .parse::<u32>()
            .ok()
    } else {
        None
    };

    let mut locks = Vec::new();

    if let Some(target_disk_num) = drive_num {
        // Enumerate volumes and lock those on this disk
        let drives = unsafe { GetLogicalDrives() };
        for i in 0..26 {
            if (drives & (1 << i)) != 0 {
                let drive_letter = (b'A' + i as u8) as char;
                let volume_path = format!(r"\\.\{}:", drive_letter);

                // Open volume
                if let Ok(volume_file) =
                    OpenOptions::new().read(true).write(true).open(&volume_path)
                {
                    let handle = HANDLE(volume_file.as_raw_handle() as _);
                    // Check if it's on our disk
                    if let Ok(disk_num) = get_volume_disk_number(handle) {
                        if disk_num == target_disk_num {
                            // Lock and dismount
                            if lock_and_dismount_volume(handle) {
                                locks.push(volume_file);
                            }
                        }
                    }
                }
            }
        }
    }

    // Open the physical drive
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .context("Failed to open device")?;

    Ok(DeviceHandle {
        file,
        _locks: locks,
    })
}

#[cfg(target_os = "windows")]
fn get_volume_disk_number(handle: HANDLE) -> Result<u32> {
    use std::mem;
    use windows::Win32::System::IO::DeviceIoControl;

    let mut sdn: STORAGE_DEVICE_NUMBER = unsafe { mem::zeroed() };
    let mut bytes_returned = 0;

    unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_GET_DEVICE_NUMBER,
            None,
            0,
            Some(&mut sdn as *mut _ as _),
            mem::size_of::<STORAGE_DEVICE_NUMBER>() as u32,
            Some(&mut bytes_returned),
            None,
        )?;
    }

    Ok(sdn.DeviceNumber)
}

#[cfg(target_os = "windows")]
fn lock_and_dismount_volume(handle: HANDLE) -> bool {
    use windows::Win32::System::IO::DeviceIoControl;
    let mut bytes_returned = 0;

    unsafe {
        // Try to lock the volume
        if DeviceIoControl(
            handle,
            FSCTL_LOCK_VOLUME,
            None,
            0,
            None,
            0,
            Some(&mut bytes_returned),
            None,
        )
        .is_err()
        {
            return false;
        }

        // Dismount the volume
        let _ = DeviceIoControl(
            handle,
            FSCTL_DISMOUNT_VOLUME,
            None,
            0,
            None,
            0,
            Some(&mut bytes_returned),
            None,
        );
    }

    true
}
