# HDDLLFRT Usage Guide

## Table of Contents

1. [Installation](#installation)
2. [Running the Tool](#running-the-tool)
3. [Operations](#operations)
4. [Safety Guidelines](#safety-guidelines)
5. [Troubleshooting](#troubleshooting)

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/DevNergis/HDDLLFRT.git
cd HDDLLFRT

# Build the release version
cargo build --release

# The binary will be in target/release/
```

### Binary Location

- Linux/macOS: `target/release/hddllfrt`
- Windows: `target\release\hddllfrt.exe`

## Running the Tool

### Linux

```bash
# Must run with sudo for device access
sudo ./target/release/hddllfrt
```

### macOS

```bash
# Must run with sudo for device access
sudo ./target/release/hddllfrt
```

### Windows

1. Right-click on Command Prompt or PowerShell
2. Select "Run as Administrator"
3. Navigate to the directory
4. Run: `.\target\release\hddllfrt.exe`

## Operations

### 1. List Devices

Lists all detected storage devices with their information:
- Model name
- Device path
- Capacity
- Type (HDD, SSD, NVMe, USB)
- Interface (SATA, NVMe, USB)
- Removable status

**Usage:**
1. Select "List Devices" from the main menu
2. Review the device list

**Example Output:**
```
Available Storage Devices:
────────────────────────────────────────────────────────────────────────────────

[1] Samsung SSD 850 EVO 500GB
    Path:      /dev/sda
    Capacity:  465.76 GB
    Type:      SSD
    Interface: SATA
    Removable: No

[2] WD My Passport 0820
    Path:      /dev/sdb
    Capacity:  931.51 GB
    Type:      USB
    Interface: USB
    Removable: Yes
```

### 2. Low Level Format

Performs a complete zero-fill format of the entire device.

**What it does:**
- Writes zeros to every sector of the device
- Takes significant time (hours for large drives)
- Permanently erases all data
- Provides real-time progress updates

**Usage:**
1. Select "Low Level Format" from main menu
2. Review and select the target device
3. Read and confirm the warnings (2 confirmations required)
4. Wait for the operation to complete

**Estimated Time:**
- 100GB HDD: ~30-60 minutes
- 500GB HDD: 2-4 hours
- 1TB HDD: 4-8 hours
- SSD: Generally faster due to higher write speeds

### 3. Quick Format

Performs a fast format by only writing to the beginning and end of the device.

**What it does:**
- Writes 10MB of zeros to the start
- Writes 10MB of zeros to the end
- Much faster than full format
- Clears partition tables and file system headers

**Usage:**
1. Select "Quick Format" from main menu
2. Review and select the target device
3. Read and confirm the warnings (2 confirmations required)
4. Operation completes in seconds

**Use Cases:**
- Quick removal of partition tables
- Fast preparation for repartitioning
- When full data erasure is not required

### 4. Secure Erase

Performs multiple passes with different patterns for secure data erasure.

**What it does:**
- Writes multiple patterns to the entire device
- Uses 4 different patterns (0x00, 0xFF, 0xAA, 0x55)
- Each pass writes to the entire device
- Suitable for secure data destruction

**Usage:**
1. Select "Secure Erase" from main menu
2. Review and select the target device
3. Enter the number of passes (1-10)
4. Read and confirm the warnings (2 confirmations required)
5. Wait for all passes to complete

**Estimated Time:**
- Time = (Low Level Format Time) × (Number of Passes)
- Example: 1TB HDD with 3 passes = 12-24 hours

**Recommended Passes:**
- Standard: 1-3 passes
- High security: 5-7 passes
- Maximum security: 10 passes

### 5. Verify Device

Reads all sectors to verify device integrity.

**What it does:**
- Reads every sector of the device
- Detects read errors or bad sectors
- Reports success or failure

**Usage:**
1. Select "Verify Device" from main menu
2. Review and select the target device
3. Wait for verification to complete

**Use Cases:**
- Check device health after format
- Detect bad sectors
- Verify device is readable

### 6. View S.M.A.R.T. Data

Displays Self-Monitoring, Analysis, and Reporting Technology data.

**What it does:**
- Shows device health status
- Displays temperature (if available)
- Shows power-on hours (if available)
- Lists S.M.A.R.T. attributes (future implementation)

**Usage:**
1. Select "View S.M.A.R.T. Data" from main menu
2. Review and select the target device
3. Review the displayed information

**Note:** Full S.M.A.R.T. attribute reading requires additional implementation and may need external tools like `smartctl`.

## Safety Guidelines

### Before Starting

1. ✅ **Backup all important data** - Operations are irreversible
2. ✅ **Verify device selection** - Double-check you selected the correct device
3. ✅ **Unmount all partitions** - Ensure device is not in use
4. ✅ **Close other programs** - Avoid conflicts with other software
5. ✅ **Use stable power** - Ensure device won't lose power during operation
6. ✅ **Run with admin privileges** - Required for device access

### During Operation

1. ⚠️ **Don't interrupt the process** - Let operations complete
2. ⚠️ **Don't power off** - Maintain stable power
3. ⚠️ **Don't disconnect** - Keep USB devices connected
4. ⚠️ **Monitor progress** - Watch for errors

### Device Selection

Common device paths:

**Linux:**
- `/dev/sda`, `/dev/sdb`, etc. - SATA/SCSI devices
- `/dev/nvme0n1`, `/dev/nvme1n1`, etc. - NVMe devices
- `/dev/mmcblk0`, `/dev/mmcblk1`, etc. - SD cards

**Windows:**
- `\\.\PhysicalDrive0`, `\\.\PhysicalDrive1`, etc.

**macOS:**
- `/dev/disk0`, `/dev/disk1`, etc.

**Warning:** Always verify you selected the correct device!

## Troubleshooting

### Permission Denied

**Problem:** Cannot access device
**Solution:** 
- Linux/macOS: Run with `sudo`
- Windows: Run as Administrator

### Device is Mounted

**Problem:** Cannot format mounted device
**Solution:**
- Linux: `sudo umount /dev/sdX*` (replace X with your device)
- macOS: `diskutil unmountDisk /dev/diskX`
- Windows: Use Disk Management to unmount volumes

### Device Not Detected

**Problem:** Device not shown in list
**Solution:**
1. Check device is connected
2. Check device is recognized by OS
3. Run with elevated privileges
4. Check device is not a loop/ram device (Linux)

### Operation Too Slow

**Problem:** Format taking extremely long
**Solution:**
1. Use Quick Format if full erasure not needed
2. Check device health (may have bad sectors)
3. Consider Secure Erase with fewer passes
4. USB devices are typically slower

### Error During Format

**Problem:** Operation fails with error
**Solution:**
1. Check device health with Verify operation
2. Check for bad sectors
3. Try a different device
4. Check system logs for details

### S.M.A.R.T. Data Unavailable

**Problem:** No S.M.A.R.T. data shown
**Solution:**
- Current implementation is basic
- Use external tools like `smartctl` for detailed data
- Some devices don't support S.M.A.R.T.

## Examples

### Example 1: Format a USB Drive

```bash
# 1. Run the tool
sudo ./hddllfrt

# 2. Select "List Devices" to find your USB drive
# 3. Note the device path (e.g., /dev/sdb)
# 4. Select "Low Level Format"
# 5. Select your USB drive
# 6. Confirm twice
# 7. Wait for completion
```

### Example 2: Securely Erase a Drive

```bash
# 1. Run the tool
sudo ./hddllfrt

# 2. Select "Secure Erase"
# 3. Select the target device
# 4. Enter 3 for number of passes
# 5. Confirm twice
# 6. Wait for all 3 passes to complete
```

### Example 3: Verify Device Health

```bash
# 1. Run the tool
sudo ./hddllfrt

# 2. Select "Verify Device"
# 3. Select the device to verify
# 4. Wait for verification to complete
# 5. Check the result (Pass/Fail)
```

## Best Practices

1. **Always backup first** - No data recovery after format
2. **Test on non-critical devices** - Familiarize yourself with the tool
3. **Verify device selection** - Check multiple times before confirming
4. **Use appropriate operation** - Choose based on your needs:
   - Quick Format: Fast, removes partitions
   - Low Level Format: Complete erasure, takes time
   - Secure Erase: Maximum security, very slow
5. **Monitor the process** - Watch for errors or issues
6. **Verify after format** - Use Verify operation to check results

## Getting Help

If you encounter issues:

1. Check this guide
2. Review README.md
3. Check the GitHub issues
4. Open a new issue with:
   - Operating system and version
   - Rust version (`rustc --version`)
   - Error messages
   - Steps to reproduce

## Warning

⚠️ **THIS TOOL PERFORMS DESTRUCTIVE OPERATIONS**

- All data will be permanently erased
- Operations cannot be undone
- There is no data recovery possible
- Always verify device selection
- Always backup important data

USE AT YOUR OWN RISK!
