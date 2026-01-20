use std::fmt;

pub mod nvme;
pub mod operations;

/// Represents a storage device
#[derive(Debug, Clone)]
pub struct StorageDevice {
    /// Device path (e.g., /dev/sda, \\.\PhysicalDrive0)
    pub path: String,
    /// Device model name
    pub model: String,
    /// Serial number
    pub serial: String,
    /// Total capacity in bytes
    pub capacity: u64,
    /// Device type (HDD, SSD, NVMe, USB, etc.)
    pub device_type: DeviceType,
    /// Whether the device is removable
    pub is_removable: bool,
    /// Interface type (SATA, NVMe, USB, etc.)
    pub interface: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceType {
    HDD,
    SSD,
    NVMe,
    USB,
    Unknown,
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::HDD => write!(f, "HDD"),
            DeviceType::SSD => write!(f, "SSD"),
            DeviceType::NVMe => write!(f, "NVMe"),
            DeviceType::USB => write!(f, "USB"),
            DeviceType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl StorageDevice {
    /// Format capacity to human-readable string
    pub fn format_capacity(&self) -> String {
        format_bytes(self.capacity)
    }

    /// Get a display name for the device
    pub fn display_name(&self) -> String {
        format!(
            "{} ({}, {})",
            self.model,
            self.format_capacity(),
            self.device_type
        )
    }
}

/// Format bytes to human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let unit_index = (bytes_f.log2() / 10.0).floor() as usize;
    let unit_index = unit_index.min(UNITS.len() - 1);

    let value = bytes_f / (1024_f64.powi(unit_index as i32));

    format!("{:.2} {}", value, UNITS[unit_index])
}

/// S.M.A.R.T. attribute
#[derive(Debug, Clone)]
pub struct SmartAttribute {
    #[allow(dead_code)]
    pub id: u8,
    pub name: String,
    pub value: u8,
    pub worst: u8,
    pub threshold: u8,
    pub raw_value: u64,
}

/// S.M.A.R.T. data for a device
#[derive(Debug, Clone)]
pub struct SmartData {
    pub attributes: Vec<SmartAttribute>,
    pub health_status: HealthStatus,
    pub temperature: Option<i32>,
    pub power_on_hours: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Good,
    Warning,
    #[allow(dead_code)]
    Critical,
    Unknown,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HealthStatus::Good => write!(f, "Good"),
            HealthStatus::Warning => write!(f, "Warning"),
            HealthStatus::Critical => write!(f, "Critical"),
            HealthStatus::Unknown => write!(f, "Unknown"),
        }
    }
}
