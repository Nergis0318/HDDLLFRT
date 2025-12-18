use super::StorageDevice;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{Read, Seek, SeekFrom, Write};

/// Callback for progress updates during format operations
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send>;

/// Perform a low-level format (zero-fill) of the device
pub fn low_level_format(
    device: &StorageDevice,
    progress_callback: Option<ProgressCallback>,
) -> Result<()> {
    erase_with_pattern(device, 0x00, progress_callback)
}

/// Perform a quick format (only write zeros to the beginning and end)
pub fn quick_format(device: &StorageDevice) -> Result<()> {
    let mut file = crate::platform::open_device_exclusive(&device.path)
        .context(format!("Failed to open device: {}", device.path))?;

    // Buffer size (10 MB)
    const BUFFER_SIZE: usize = 10 * 1024 * 1024;
    let buffer = vec![0u8; BUFFER_SIZE];

    // Write to the beginning
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&buffer)?;

    // Write to the end (if device is larger than buffer)
    if device.capacity > BUFFER_SIZE as u64 {
        let seek_pos = device.capacity - (BUFFER_SIZE as u64);
        file.seek(SeekFrom::Start(seek_pos))?;
        file.write_all(&buffer)?;
    }

    file.sync_all()?;

    Ok(())
}

/// Verify device by reading all sectors
pub fn verify_device(
    device: &StorageDevice,
    progress_callback: Option<ProgressCallback>,
) -> Result<bool> {
    let mut handle = crate::platform::open_device_exclusive(&device.path)
        .context(format!("Failed to open device: {}", device.path))?;

    let total_bytes = device.capacity;

    // Use a larger buffer (4MB)
    const BUFFER_SIZE: usize = 4 * 1024 * 1024;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    let mut current = 0;
    handle.seek(SeekFrom::Start(0))?;

    while current < total_bytes {
        let remaining = total_bytes - current;
        let to_read = std::cmp::min(BUFFER_SIZE as u64, remaining) as usize;

        match handle.read(&mut buffer[..to_read]) {
            Ok(n) => {
                if n == 0 {
                    break;
                } // EOF
                current += n as u64;
                if let Some(ref cb) = progress_callback {
                    cb(current, total_bytes);
                }
            }
            Err(e) => {
                eprintln!("Read error at byte {}: {}", current, e);
                return Ok(false);
            }
        }
    }

    Ok(true)
}

/// Erase device with a specific pattern
pub fn erase_with_pattern(
    device: &StorageDevice,
    pattern: u8,
    progress_callback: Option<ProgressCallback>,
) -> Result<()> {
    let mut handle = crate::platform::open_device_exclusive(&device.path)
        .context(format!("Failed to open device: {}", device.path))?;

    let total_bytes = device.capacity;

    // Use a larger buffer (4MB) for better performance
    const BUFFER_SIZE: usize = 4 * 1024 * 1024;
    let buffer = vec![pattern; BUFFER_SIZE];

    let mut current = 0;

    // Ensure we start at the beginning
    handle.seek(SeekFrom::Start(0))?;

    while current < total_bytes {
        let remaining = total_bytes - current;
        let to_write = std::cmp::min(BUFFER_SIZE as u64, remaining) as usize;

        handle.write_all(&buffer[..to_write])?;
        current += to_write as u64;

        if let Some(ref callback) = progress_callback {
            callback(current, total_bytes);
        }
    }

    handle.sync_all()?;

    Ok(())
}

/// Perform a secure erase (multiple passes with different patterns)
pub fn secure_erase(device: &StorageDevice, passes: u32) -> Result<()> {
    let patterns = vec![0x00, 0xFF, 0xAA, 0x55];

    for pass in 0..passes {
        let pattern = patterns[(pass as usize) % patterns.len()];
        println!(
            "Pass {}/{}: Writing pattern 0x{:02X}...",
            pass + 1,
            passes,
            pattern
        );

        let pb = ProgressBar::new(device.capacity);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .expect("Invalid progress bar template")
                .progress_chars("#>-"),
        );

        let pb_clone = pb.clone();
        erase_with_pattern(
            device,
            pattern,
            Some(Box::new(move |current, _total| {
                pb_clone.set_position(current);
            })),
        )?;

        pb.finish_with_message(format!("Pass {} completed", pass + 1));
    }

    Ok(())
}
