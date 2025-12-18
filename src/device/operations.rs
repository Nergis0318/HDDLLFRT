use super::StorageDevice;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};

/// Callback for progress updates during format operations
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send>;

/// Perform a low-level format (zero-fill) of the device
pub fn low_level_format(
    device: &StorageDevice,
    progress_callback: Option<ProgressCallback>,
) -> Result<()> {
    // Open device with write permissions
    let mut file = OpenOptions::new()
        .write(true)
        .open(&device.path)
        .context(format!("Failed to open device: {}", device.path))?;

    // Buffer size for writing (1 MB)
    const BUFFER_SIZE: usize = 1024 * 1024;
    let buffer = vec![0u8; BUFFER_SIZE];

    let total_bytes = device.capacity;
    let mut written_bytes: u64 = 0;

    // Seek to the beginning
    file.seek(SeekFrom::Start(0))?;

    // Write zeros to the entire device
    while written_bytes < total_bytes {
        let remaining = total_bytes - written_bytes;
        let to_write = BUFFER_SIZE.min(remaining as usize);

        file.write_all(&buffer[..to_write])
            .context("Failed to write to device")?;

        written_bytes += to_write as u64;

        // Call progress callback if provided
        if let Some(ref callback) = progress_callback {
            callback(written_bytes, total_bytes);
        }
    }

    // Flush to ensure all data is written
    file.sync_all().context("Failed to sync device")?;

    Ok(())
}

/// Perform a quick format (only write zeros to the beginning and end)
pub fn quick_format(device: &StorageDevice) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .open(&device.path)
        .context(format!("Failed to open device: {}", device.path))?;

    // Buffer size (10 MB)
    const BUFFER_SIZE: usize = 10 * 1024 * 1024;
    let buffer = vec![0u8; BUFFER_SIZE];

    // Write to the beginning
    file.seek(SeekFrom::Start(0))?;
    file.write_all(&buffer)?;

    // Write to the end (if device is larger than buffer)
    if device.capacity > BUFFER_SIZE as u64 {
        file.seek(SeekFrom::End(-(BUFFER_SIZE as i64)))?;
        file.write_all(&buffer)?;
    }

    file.sync_all()?;

    Ok(())
}

/// Verify device by reading all sectors
pub fn verify_device(device: &StorageDevice) -> Result<bool> {
    use std::io::Read;

    let mut file = OpenOptions::new()
        .read(true)
        .open(&device.path)
        .context(format!("Failed to open device: {}", device.path))?;

    const BUFFER_SIZE: usize = 1024 * 1024;
    let mut buffer = vec![0u8; BUFFER_SIZE];

    let total_bytes = device.capacity;
    let mut read_bytes: u64 = 0;

    while read_bytes < total_bytes {
        let remaining = total_bytes - read_bytes;
        let to_read = BUFFER_SIZE.min(remaining as usize);

        match file.read(&mut buffer[..to_read]) {
            Ok(0) => break, // EOF
            Ok(n) => read_bytes += n as u64,
            Err(e) => {
                eprintln!("Read error at byte {}: {}", read_bytes, e);
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
    let mut file = OpenOptions::new()
        .write(true)
        .open(&device.path)
        .context(format!("Failed to open device: {}", device.path))?;

    const BUFFER_SIZE: usize = 1024 * 1024;
    let buffer = vec![pattern; BUFFER_SIZE];

    let total_bytes = device.capacity;
    let mut written_bytes: u64 = 0;

    file.seek(SeekFrom::Start(0))?;

    while written_bytes < total_bytes {
        let remaining = total_bytes - written_bytes;
        let to_write = BUFFER_SIZE.min(remaining as usize);

        file.write_all(&buffer[..to_write])
            .context("Failed to write to device")?;

        written_bytes += to_write as u64;

        if let Some(ref callback) = progress_callback {
            callback(written_bytes, total_bytes);
        }
    }

    file.sync_all()?;

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
