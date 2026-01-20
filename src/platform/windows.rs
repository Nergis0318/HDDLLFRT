use crate::device::{DeviceType, HealthStatus, SmartData, StorageDevice};
use crate::platform::DeviceHandle;
use anyhow::{Context, Result};

#[cfg(target_os = "windows")]
use windows::core::PCWSTR;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HANDLE;
#[cfg(target_os = "windows")]
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetLogicalDrives, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::Ioctl::IOCTL_DISK_GET_DRIVE_GEOMETRY_EX;
#[cfg(target_os = "windows")]
use windows::Win32::System::Ioctl::{
    PropertyStandardQuery, StorageDeviceProperty, StorageDeviceSeekPenaltyProperty,
    DEVICE_SEEK_PENALTY_DESCRIPTOR, FSCTL_DISMOUNT_VOLUME, FSCTL_LOCK_VOLUME, GETVERSIONINPARAMS,
    IOCTL_STORAGE_GET_DEVICE_NUMBER, IOCTL_STORAGE_QUERY_PROPERTY, SENDCMDINPARAMS,
    SENDCMDOUTPARAMS, SMART_GET_VERSION, SMART_RCV_DRIVE_DATA, STORAGE_DEVICE_DESCRIPTOR,
    STORAGE_DEVICE_NUMBER, STORAGE_PROPERTY_QUERY,
};
// #[cfg(target_os = "windows")]
// use windows::Win32::System::SystemServices::{GENERIC_READ, GENERIC_WRITE};

const GENERIC_READ: u32 = 0x80000000;
const GENERIC_WRITE: u32 = 0x40000000;

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

    // Try to open physical drives 0-15
    for i in 0..16 {
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
struct DeviceDetails {
    model: Option<String>,
    serial: Option<String>,
    interface: String,
    device_type: DeviceType,
    is_removable: bool,
}

#[cfg(target_os = "windows")]
unsafe fn get_device_details(handle: HANDLE) -> DeviceDetails {
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
            } else {
                if device_type == DeviceType::Unknown {
                    device_type = DeviceType::HDD;
                }
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

pub fn read_smart_data(device: &StorageDevice) -> Result<SmartData> {
    #[cfg(target_os = "windows")]
    return read_smart_data_impl(device);

    #[cfg(not(target_os = "windows"))]
    Ok(SmartData {
        attributes: Vec::new(),
        health_status: HealthStatus::Unknown,
        temperature: None,
        power_on_hours: None,
    })
}

#[cfg(target_os = "windows")]
fn read_smart_data_impl(device: &StorageDevice) -> Result<SmartData> {
    use crate::device::SmartAttribute;
    use std::mem;
    use widestring::U16CString;
    use windows::core::PCWSTR;
    use windows::Win32::System::IO::DeviceIoControl;

    let wide_path = U16CString::from_str(&device.path)?;

    unsafe {
        let handle = CreateFileW(
            PCWSTR(wide_path.as_ptr()),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            Default::default(),
            None,
        );

        if handle.is_err() || handle.as_ref().unwrap().is_invalid() {
            anyhow::bail!("Failed to open device for SMART data");
        }
        let handle = handle.unwrap();

        // Handle NVMe devices
        if device.device_type == DeviceType::NVMe {
            let result = read_nvme_smart_data_internal(handle);
            let _ = windows::Win32::Foundation::CloseHandle(handle);
            return result;
        }

        // Check SMART version/support
        let mut version_params: GETVERSIONINPARAMS = mem::zeroed();
        let mut bytes_returned: u32 = 0;

        let result = DeviceIoControl(
            handle,
            SMART_GET_VERSION,
            None,
            0,
            Some(&mut version_params as *mut _ as *mut _),
            mem::size_of::<GETVERSIONINPARAMS>() as u32,
            Some(&mut bytes_returned),
            None,
        );

        if result.is_err() {
            let _ = windows::Win32::Foundation::CloseHandle(handle);
            anyhow::bail!("SMART not supported or failed to get version");
        }

        // Read Attributes
        let mut cmd_in: SENDCMDINPARAMS = mem::zeroed();
        cmd_in.cBufferSize = 512;
        cmd_in.irDriveRegs.bFeaturesReg = 0xD0; // READ_ATTRIBUTE_VALUES
        cmd_in.irDriveRegs.bSectorCountReg = 1;
        cmd_in.irDriveRegs.bSectorNumberReg = 1;
        cmd_in.irDriveRegs.bCylLowReg = 0x4F; // SMART
        cmd_in.irDriveRegs.bCylHighReg = 0xC2; // SMART
        cmd_in.irDriveRegs.bDriveHeadReg = 0xA0 | ((version_params.bIDEDeviceMap >> 4) & 0x10);
        cmd_in.irDriveRegs.bCommandReg = 0xB0; // SMART COMMAND

        const OUT_BUFFER_SIZE: usize = mem::size_of::<SENDCMDOUTPARAMS>() + 512;
        let mut buffer = [0u8; OUT_BUFFER_SIZE];

        let result = DeviceIoControl(
            handle,
            SMART_RCV_DRIVE_DATA,
            Some(&cmd_in as *const _ as *const _),
            mem::size_of::<SENDCMDINPARAMS>() as u32,
            Some(buffer.as_mut_ptr() as *mut _),
            OUT_BUFFER_SIZE as u32,
            Some(&mut bytes_returned),
            None,
        );

        if result.is_err() {
            let _ = windows::Win32::Foundation::CloseHandle(handle);
            anyhow::bail!("Failed to read SMART attributes");
        }

        // Parse attributes
        // buffer layout: SENDCMDOUTPARAMS (header) + 512 bytes (data)
        // SENDCMDOUTPARAMS: cBufferSize(4) + DriverStatus(12) + bBuffer(1) + padding
        // We want the data starting at bBuffer.
        // Offset is 4 + 12 = 16.
        let data_offset = 16;
        let smart_data_buffer = buffer.as_ptr().add(data_offset);

        let mut attributes = Vec::new();
        let mut temperature = None;
        let mut power_on_hours = None;

        // SMART data structure:
        // 0-1: Version
        // 2-361: Attributes (30 * 12 bytes)

        for i in 0..30 {
            let offset = 2 + i * 12;
            let id = *smart_data_buffer.add(offset);
            if id == 0 {
                continue;
            }

            let current = *smart_data_buffer.add(offset + 3);
            let worst = *smart_data_buffer.add(offset + 4);

            let mut raw_value: u64 = 0;
            for j in 0..6 {
                raw_value |= (*smart_data_buffer.add(offset + 5 + j) as u64) << (j * 8);
            }

            attributes.push(SmartAttribute {
                id,
                name: get_attribute_name(id),
                value: current,
                worst,
                threshold: 0,
                raw_value,
            });

            if id == 194 || id == 190 {
                // Temperature
                temperature = Some((raw_value & 0xFF) as i32);
            }
            if id == 9 {
                // Power on hours
                power_on_hours = Some(raw_value);
            }
        }

        let _ = windows::Win32::Foundation::CloseHandle(handle);

        Ok(SmartData {
            attributes,
            health_status: HealthStatus::Unknown,
            temperature,
            power_on_hours,
        })
    }
}

fn get_attribute_name(id: u8) -> String {
    let name = match id {
        1 => "Read Error Rate",
        2 => "Throughput Performance",
        3 => "Spin-Up Time",
        4 => "Start/Stop Count",
        5 => "Reallocated Sectors Count",
        7 => "Seek Error Rate",
        8 => "Seek Time Performance",
        9 => "Power-On Hours",
        10 => "Spin Retry Count",
        12 => "Power Cycle Count",
        187 => "Reported Uncorrectable Errors",
        188 => "Command Timeout",
        190 => "Airflow Temperature",
        191 => "G-Sense Error Rate",
        192 => "Power-Off Retract Count",
        193 => "Load Cycle Count",
        194 => "Temperature",
        195 => "Hardware ECC Recovered",
        196 => "Reallocation Event Count",
        197 => "Current Pending Sector Count",
        198 => "Offline Uncorrectable Sector Count",
        199 => "UDMA CRC Error Count",
        200 => "Multi-Zone Error Rate",
        240 => "Head Flying Hours",
        241 => "Total LBAs Written",
        242 => "Total LBAs Read",
        _ => return format!("Unknown Attribute {}", id),
    };
    name.to_string()
}

pub fn is_device_mounted(_device: &StorageDevice) -> Result<bool> {
    // On Windows, check if any volume is using this physical drive
    // This is a simplified implementation
    Ok(false)
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
        if DeviceIoControl(
            handle,
            FSCTL_DISMOUNT_VOLUME,
            None,
            0,
            None,
            0,
            Some(&mut bytes_returned),
            None,
        )
        .is_err()
        {
            // If dismount fails, we might still have the lock, which is good.
        }
    }

    true
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct STORAGE_PROTOCOL_SPECIFIC_DATA {
    ProtocolType: u32,
    DataType: u32,
    ProtocolDataRequestValue: u32,
    ProtocolDataRequestSubValue: u32,
    ProtocolDataOffset: u32,
    ProtocolDataLength: u32,
    FixedProtocolReturnData: u32,
    ProtocolDataRequestSubValue2: u32,
    Reserved: [u32; 3],
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct StoragePropertyQueryRequest {
    PropertyId: u32,
    QueryType: u32,
    ProtocolSpecific: STORAGE_PROTOCOL_SPECIFIC_DATA,
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct StorageProtocolDataDescriptor {
    Version: u32,
    Size: u32,
    ProtocolDataOffset: u32,
    ProtocolDataLength: u32,
}

#[cfg(target_os = "windows")]
unsafe fn read_nvme_smart_data_internal(handle: HANDLE) -> Result<SmartData> {
    use crate::device::nvme::NvmeSmartLog;
    use std::mem;
    use windows::Win32::System::IO::DeviceIoControl;

    // Constants
    const StorageAdapterProtocolSpecificProperty: u32 = 49;
    const PropertyStandardQuery: u32 = 0;
    const ProtocolTypeNvme: u32 = 2;
    const NVMeDataTypeLogPage: u32 = 3;
    const NVMeLogPageHealthInfo: u32 = 2;

    let mut request: StoragePropertyQueryRequest = mem::zeroed();

    request.PropertyId = StorageAdapterProtocolSpecificProperty;
    request.QueryType = PropertyStandardQuery;

    request.ProtocolSpecific.ProtocolType = ProtocolTypeNvme;
    request.ProtocolSpecific.DataType = NVMeDataTypeLogPage;
    request.ProtocolSpecific.ProtocolDataRequestValue = NVMeLogPageHealthInfo;
    request.ProtocolSpecific.ProtocolDataRequestSubValue = 0; // Lower 32 bits of offset (0 for Global Log)
    request.ProtocolSpecific.ProtocolDataOffset =
        mem::size_of::<STORAGE_PROTOCOL_SPECIFIC_DATA>() as u32;
    request.ProtocolSpecific.ProtocolDataLength = 512; // Expected length

    // Buffer for Result
    // Header (approx 16 bytes) + 512 bytes data
    let mut buffer = [0u8; 1024];
    let mut bytes_returned = 0;

    let result = DeviceIoControl(
        handle,
        IOCTL_STORAGE_QUERY_PROPERTY,
        Some(&request as *const _ as *const _),
        mem::size_of::<StoragePropertyQueryRequest>() as u32,
        Some(buffer.as_mut_ptr() as *mut _),
        buffer.len() as u32,
        Some(&mut bytes_returned),
        None,
    );

    if result.is_err() {
        // Fallback or error
        anyhow::bail!("Failed to query NVMe SMART data (IOCTL failed)");
    }

    // Parse result
    let descriptor = &*(buffer.as_ptr() as *const StorageProtocolDataDescriptor);

    if descriptor.ProtocolDataLength < 512 {
        anyhow::bail!("Returned NVMe log data too short");
    }

    if descriptor.ProtocolDataOffset as usize + 512 > buffer.len() {
        anyhow::bail!("NVMe data offset out of bounds");
    }

    let data_ptr = buffer.as_ptr().add(descriptor.ProtocolDataOffset as usize);
    let slice = std::slice::from_raw_parts(data_ptr, 512);

    let log = NvmeSmartLog::parse(slice).context("Invalid NVMe Log Page")?;

    Ok(log.to_smart_data())
}
