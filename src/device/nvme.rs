use crate::device::{HealthStatus, SmartAttribute, SmartData};

/// NVMe Admin Command Opcodes
pub const NVME_ADMIN_GET_LOG_PAGE: u8 = 0x02;
#[allow(dead_code)]
pub const NVME_ADMIN_IDENTIFY: u8 = 0x06;

/// NVMe Log Identifiers
#[allow(dead_code)]
pub const NVME_LOG_ERROR_INFO: u8 = 0x01;
pub const NVME_LOG_SMART_HEALTH: u8 = 0x02;
#[allow(dead_code)]
pub const NVME_LOG_FW_SLOT: u8 = 0x03;

/// NVMe Smart Log Page (512 bytes)
/// Ref: NVM Express Base Specification, Section 5.14.1.2
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct NvmeSmartLog {
    pub critical_warning: u8,
    pub composite_temp: [u8; 2],
    pub available_spare: u8,
    pub available_spare_threshold: u8,
    pub percent_used: u8,
    pub endurance_group_critical_warning_summary: u8,
    pub reserved1: [u8; 25],
    pub data_units_read: [u8; 16], // u128, little endian (1000 units of 512 bytes)
    pub data_units_written: [u8; 16], // u128, little endian
    pub host_read_commands: [u8; 16],
    pub host_write_commands: [u8; 16],
    pub controller_busy_time: [u8; 16],
    pub power_cycles: [u8; 16],
    pub power_on_hours: [u8; 16],
    pub unsafe_shutdowns: [u8; 16],
    pub media_errors: [u8; 16],
    pub num_error_info_log_entries: [u8; 16],
    pub warning_composite_temp_time: u32,
    pub critical_composite_temp_time: u32,
    pub temp_sensor_1: u16,
    pub temp_sensor_2: u16,
    pub temp_sensor_3: u16,
    pub temp_sensor_4: u16,
    pub temp_sensor_5: u16,
    pub temp_sensor_6: u16,
    pub temp_sensor_7: u16,
    pub temp_sensor_8: u16,
    pub thermal_management_temp1_transition_count: u32,
    pub thermal_management_temp2_transition_count: u32,
    pub thermal_management_temp1_total_time: u32,
    pub thermal_management_temp2_total_time: u32,
    pub reserved2: [u8; 280],
}

impl NvmeSmartLog {
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 512 {
            return None;
        }

        // Safety: We verified the length. The struct is packed and Copy.
        // In Rust, we need to be careful with transmuting bytes to packed structs directly
        // due to alignment, but here we'll do a manual copy or safe cast if possible.
        // A safer way in Rust without unsafe transmute (which is tricky with endianness anyway)
        // is to read fields manually or use the `bytemuck` crate if available.
        // Since we don't want to add deps, we'll use `unsafe` with a pointer cast
        // assuming the input buffer is valid.

        let ptr = data.as_ptr() as *const NvmeSmartLog;
        unsafe { Some(*ptr) }
    }

    pub fn to_smart_data(&self) -> SmartData {
        let mut attributes = Vec::new();

        // Helper to convert u128 le bytes to u128
        let to_u128 = |bytes: [u8; 16]| -> u128 { u128::from_le_bytes(bytes) };

        // Critical Warning
        attributes.push(SmartAttribute {
            id: 1,
            name: "Critical Warning".to_string(),
            value: 0, // NVMe raw values don't map directly to "normalized" values like ATA
            worst: 0,
            threshold: 0,
            raw_value: self.critical_warning as u64,
        });

        // Temperature (Kelvin)
        let temp_k = u16::from_le_bytes(self.composite_temp);
        let temp_c = if temp_k >= 273 {
            temp_k as i32 - 273
        } else {
            0
        };

        attributes.push(SmartAttribute {
            id: 2,
            name: "Temperature (C)".to_string(),
            value: 0,
            worst: 0,
            threshold: 0,
            raw_value: temp_c as u64,
        });

        // Available Spare
        attributes.push(SmartAttribute {
            id: 3,
            name: "Available Spare (%)".to_string(),
            value: self.available_spare,
            worst: 0,
            threshold: self.available_spare_threshold,
            raw_value: self.available_spare as u64,
        });

        // Percentage Used
        attributes.push(SmartAttribute {
            id: 4,
            name: "Percentage Used (%)".to_string(),
            value: 100u8.saturating_sub(self.percent_used), // Invert for "Health" look? Or just raw.
            worst: 0,
            threshold: 0,
            raw_value: self.percent_used as u64,
        });

        // Data Units Read
        let units_read = to_u128(self.data_units_read);
        attributes.push(SmartAttribute {
            id: 5,
            name: "Data Units Read".to_string(),
            value: 0,
            worst: 0,
            threshold: 0,
            raw_value: (units_read & 0xFFFFFFFFFFFFFFFF) as u64, // Truncate for display if needed
        });

        // Data Units Written
        let units_written = to_u128(self.data_units_written);
        attributes.push(SmartAttribute {
            id: 6,
            name: "Data Units Written".to_string(),
            value: 0,
            worst: 0,
            threshold: 0,
            raw_value: (units_written & 0xFFFFFFFFFFFFFFFF) as u64,
        });

        // Power Cycles
        let power_cycles = to_u128(self.power_cycles);
        attributes.push(SmartAttribute {
            id: 7,
            name: "Power Cycles".to_string(),
            value: 0,
            worst: 0,
            threshold: 0,
            raw_value: power_cycles as u64,
        });

        // Power On Hours
        let power_hours = to_u128(self.power_on_hours);
        attributes.push(SmartAttribute {
            id: 8,
            name: "Power On Hours".to_string(),
            value: 0,
            worst: 0,
            threshold: 0,
            raw_value: power_hours as u64,
        });

        // Unsafe Shutdowns
        let unsafe_shutdowns = to_u128(self.unsafe_shutdowns);
        attributes.push(SmartAttribute {
            id: 9,
            name: "Unsafe Shutdowns".to_string(),
            value: 0,
            worst: 0,
            threshold: 0,
            raw_value: unsafe_shutdowns as u64,
        });

        // Media Errors
        let media_errors = to_u128(self.media_errors);
        attributes.push(SmartAttribute {
            id: 10,
            name: "Media and Data Integrity Errors".to_string(),
            value: 0,
            worst: 0,
            threshold: 0,
            raw_value: media_errors as u64,
        });

        // Determine Health Status
        let health_status = if self.critical_warning != 0 {
            HealthStatus::Warning // Or Critical depending on the bit
        } else if self.available_spare < self.available_spare_threshold {
            HealthStatus::Warning
        } else {
            HealthStatus::Good
        };

        SmartData {
            attributes,
            health_status,
            temperature: Some(temp_c),
            power_on_hours: Some(power_hours as u64),
        }
    }
}
