use std::fmt;

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
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let unit_index = (bytes_f.log2() / 10.0).floor() as usize;
    let unit_index = unit_index.min(UNITS.len() - 1);

    let value = bytes_f / (1024_f64.powi(unit_index as i32));

    format!("{:.2} {}", value, UNITS[unit_index])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn test_format_bytes_one_kib() {
        assert_eq!(format_bytes(1024), "1.00 KiB");
    }

    #[test]
    fn test_format_bytes_one_mib() {
        assert_eq!(format_bytes(1024 * 1024), "1.00 MiB");
    }

    #[test]
    fn test_format_bytes_one_gib() {
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GiB");
    }

    #[test]
    fn test_format_bytes_one_tib() {
        assert_eq!(format_bytes(1024_u64.pow(4)), "1.00 TiB");
    }

    #[test]
    fn test_format_bytes_500_gib() {
        assert_eq!(format_bytes(500 * 1024_u64.pow(3)), "500.00 GiB");
    }

    #[test]
    fn test_format_bytes_small() {
        assert_eq!(format_bytes(512), "512.00 B");
    }

    #[test]
    fn test_storage_device_display_name() {
        let device = StorageDevice {
            path: "/dev/sda".to_string(),
            model: "TestDrive".to_string(),
            serial: "ABC123".to_string(),
            capacity: 1024 * 1024 * 1024,
            device_type: DeviceType::HDD,
            is_removable: false,
            interface: "SATA".to_string(),
        };
        assert_eq!(device.display_name(), "TestDrive (1.00 GiB, HDD)");
    }

    #[test]
    fn test_device_type_display() {
        assert_eq!(format!("{}", DeviceType::HDD), "HDD");
        assert_eq!(format!("{}", DeviceType::SSD), "SSD");
        assert_eq!(format!("{}", DeviceType::NVMe), "NVMe");
        assert_eq!(format!("{}", DeviceType::USB), "USB");
        assert_eq!(format!("{}", DeviceType::Unknown), "Unknown");
    }

    #[test]
    fn test_device_type_equality() {
        assert_eq!(DeviceType::HDD, DeviceType::HDD);
        assert_ne!(DeviceType::HDD, DeviceType::SSD);
    }
}
