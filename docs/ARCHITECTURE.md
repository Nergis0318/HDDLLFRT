# HDDLLFRT Architecture

This document describes the architecture and design of the HDDLLFRT (HDD Low Level Format Rust Tool).

## Overview

HDDLLFRT is a cross-platform storage device low-level format tool written in Rust. It provides a safe, interactive way to perform destructive operations on storage devices across Linux, Windows, and macOS.

## Project Structure

```
HDDLLFRT/
├── src/
│   ├── main.rs              # Application entry point and main loop
│   ├── device/              # Device-related structures and operations
│   │   ├── mod.rs           # Device data structures (StorageDevice, SmartData, etc.)
│   │   └── operations.rs    # Core operations (format, erase, verify)
│   ├── platform/            # Platform-specific implementations
│   │   ├── mod.rs           # Platform abstraction layer
│   │   ├── linux.rs         # Linux-specific device detection
│   │   ├── windows.rs       # Windows-specific device detection
│   │   └── macos.rs         # macOS-specific device detection
│   └── ui/                  # User interface
│       └── mod.rs           # CLI menu system and user interactions
├── docs/                    # Documentation
│   ├── ARCHITECTURE.md      # This file
│   └── USAGE.md            # User guide
├── .github/
│   └── workflows/
│       └── ci.yml          # CI/CD configuration
├── Cargo.toml              # Project dependencies and metadata
├── CONTRIBUTING.md         # Contributing guidelines
└── README.md              # Project overview
```

## Module Design

### Main Module (`main.rs`)

**Purpose:** Application entry point and main control flow

**Responsibilities:**
- Initialize the application
- Check for administrator privileges
- Display main menu and handle user choices
- Coordinate between UI and device operations
- Handle high-level error management

**Key Functions:**
- `main()` - Entry point, initializes logger
- `run()` - Main application loop
- `handle_*()` - Handlers for each menu option

### Device Module (`device/`)

#### `device/mod.rs`

**Purpose:** Define core data structures for storage devices

**Key Types:**
- `StorageDevice` - Represents a physical storage device with metadata
  - `path`: Device path in OS-specific format
  - `model`: Device model name
  - `serial`: Serial number
  - `capacity`: Total capacity in bytes
  - `device_type`: Enum (HDD, SSD, NVMe, USB, Unknown)
  - `is_removable`: Whether device is removable
  - `interface`: Interface type (SATA, NVMe, USB)

- `DeviceType` - Enum for device types
- `SmartData` - S.M.A.R.T. health information
- `SmartAttribute` - Individual S.M.A.R.T. attribute
- `HealthStatus` - Device health status enum

**Key Functions:**
- `format_bytes()` - Convert bytes to human-readable format
- `display_name()` - Generate user-friendly device name

#### `device/operations.rs`

**Purpose:** Implement core device operations

**Key Functions:**
- `low_level_format()` - Zero-fill entire device
  - Opens device with write access
  - Writes zeros in 1MB chunks
  - Provides progress callbacks
  - Ensures data is synced to device

- `quick_format()` - Fast format (beginning and end only)
  - Writes 10MB zeros to start
  - Writes 10MB zeros to end
  - Quick way to clear partition tables

- `erase_with_pattern()` - Write specific pattern to device
  - Used by secure_erase for multiple passes
  - Supports custom byte patterns

- `secure_erase()` - Multiple-pass secure erasure
  - Uses patterns: 0x00, 0xFF, 0xAA, 0x55
  - Multiple passes for secure data destruction
  - Progress tracking per pass

- `verify_device()` - Read and verify all sectors
  - Reads entire device
  - Detects read errors
  - Reports success/failure

### Platform Module (`platform/`)

#### `platform/mod.rs`

**Purpose:** Platform abstraction layer

**Key Functions:**
- `detect_devices()` - Detect all storage devices (platform-specific)
- `has_admin_privileges()` - Check for elevated privileges
- `read_smart_data()` - Read S.M.A.R.T. data (platform-specific)
- `is_device_mounted()` - Check if device is mounted

Each function dispatches to platform-specific implementation based on target OS.

#### `platform/linux.rs`

**Linux Implementation:**

**Device Detection:**
- Reads `/sys/block/` to enumerate block devices
- Filters out loop, ram, and dm devices
- Reads device info from sysfs:
  - Model from `/sys/block/*/device/model`
  - Serial from `/sys/block/*/device/serial`
  - Size from `/sys/block/*/size` (in 512-byte sectors)
  - Rotation from `/sys/block/*/queue/rotational` (SSD vs HDD)

**Device Type Detection:**
- NVMe: Check path contains "nvme"
- SSD: rotational = 0
- HDD: rotational = 1
- USB: Check for idVendor in device tree

**Mount Detection:**
- Reads `/proc/mounts`
- Checks if device or its partitions are mounted

**Privilege Check:**
- Uses `libc::geteuid() == 0`

#### `platform/windows.rs`

**Windows Implementation:**

**Device Detection:**
- Iterates through `\\.\PhysicalDrive0` to `\\.\PhysicalDrive15`
- Uses Windows API:
  - `CreateFileW()` to open device
  - `DeviceIoControl()` with `IOCTL_DISK_GET_DRIVE_GEOMETRY_EX` for size
  - `STORAGE_PROPERTY_QUERY` for device descriptor (planned)

**Privilege Check:**
- Executes `net session` command
- Success indicates administrator privileges

#### `platform/macos.rs`

**macOS Implementation:**

**Device Detection:**
- Iterates through `/dev/disk0` to `/dev/disk19`
- Uses `diskutil info -plist` for device information
- Parses plist output for:
  - MediaName
  - TotalSize
  - Removable status
  - Device type (SSD, USB, HDD)

**Mount Detection:**
- Uses `diskutil info` to check mount status
- Looks for "Mounted: Yes" in output

**Privilege Check:**
- Uses `libc::geteuid() == 0`

### UI Module (`ui/`)

#### `ui/mod.rs`

**Purpose:** User interface and interaction handling

**Key Functions:**

- `print_banner()` - Display application banner
- `show_main_menu()` - Display and handle main menu
  - Returns `MainMenuChoice` enum

- `list_devices()` - Display formatted device list
  - Shows all device information
  - Color-coded output

- `select_device()` - Interactive device selection
  - Uses `dialoguer` for selection
  - Returns selected device index

- `confirm_dangerous_operation()` - Safety confirmation
  - Displays prominent warnings
  - Requires two separate confirmations
  - Shows device information

- `perform_low_level_format()` - Execute low-level format
  - Sets up progress bar
  - Calls device operation
  - Handles errors

- `perform_quick_format()` - Execute quick format
- `perform_secure_erase()` - Execute secure erase
- `perform_verify()` - Execute device verification
- `display_smart_data()` - Display S.M.A.R.T. information

- `check_prerequisites()` - Verify operation prerequisites
  - Check admin privileges
  - Check if device is mounted
  - Return errors if prerequisites not met

## Data Flow

### Device Detection Flow

```
main.rs:handle_list_devices()
    ↓
platform::detect_devices()
    ↓
[Platform-specific implementation]
    ↓
Returns Vec<StorageDevice>
    ↓
ui::list_devices()
    ↓
Display to user
```

### Format Operation Flow

```
main.rs:handle_low_level_format()
    ↓
platform::detect_devices()
    ↓
ui::list_devices()
    ↓
ui::select_device()
    ↓
ui::check_prerequisites()
    ├─→ platform::has_admin_privileges()
    └─→ platform::is_device_mounted()
    ↓
ui::confirm_dangerous_operation()
    ↓
ui::perform_low_level_format()
    ↓
device::operations::low_level_format()
    ├─→ Open device file
    ├─→ Write zeros in chunks
    ├─→ Update progress callback
    └─→ Sync to disk
    ↓
Display completion message
```

## Safety Features

### Multiple Confirmation Prompts

1. **First Confirmation:**
   - Display device information
   - Show prominent warnings
   - Require explicit confirmation

2. **Second Confirmation:**
   - Final confirmation with device path
   - Cannot be bypassed

### Privilege Checks

- Operations require administrator/root privileges
- Checked at startup and before operations
- Clear error messages when privileges insufficient

### Mount Detection

- Prevents formatting of mounted devices
- Checks before destructive operations
- Platform-specific implementation

### Progress Tracking

- Real-time progress updates
- Allows monitoring of long operations
- Uses progress bars for visual feedback

## Error Handling

### Strategy

- Uses `anyhow::Result<T>` for flexible error handling
- Provides context with `.context()` method
- Errors propagate up to main handlers
- User-friendly error messages

### Error Points

1. **Device Access:**
   - Permission denied
   - Device not found
   - Device busy

2. **Operation Errors:**
   - Write failures
   - Read failures
   - Sync failures

3. **Prerequisite Failures:**
   - Insufficient privileges
   - Device mounted
   - Device in use

## Dependencies

### Core Dependencies

- `clap` - Command-line argument parsing (currently not used, for future CLI args)
- `dialoguer` - Interactive prompts and menus
- `indicatif` - Progress bars and spinners
- `console` - Terminal styling and colors
- `anyhow` - Flexible error handling
- `thiserror` - Error type derivation
- `serde` / `serde_json` - Serialization (for future config files)
- `log` / `env_logger` - Logging framework

### Platform-Specific Dependencies

**Linux:**
- `libc` - C library bindings
- `nix` - Unix system APIs

**Windows:**
- `windows` - Windows API bindings
- `widestring` - Wide string handling

**macOS:**
- `core-foundation` - Core Foundation framework
- `io-kit-sys` - IOKit bindings
- `libc` - C library bindings

## Future Enhancements

### Planned Features

1. **Full S.M.A.R.T. Support:**
   - Implement ATA command passthrough
   - Read all S.M.A.R.T. attributes
   - Calculate health scores

2. **Bad Sector Management:**
   - Detect bad sectors
   - Map bad sectors
   - Report sector statistics

3. **Speed Testing:**
   - Sequential read/write benchmarks
   - Random I/O testing
   - Comparison with device specs

4. **Enhanced Device Support:**
   - Better NVMe detection
   - RAID controller support
   - Virtual disk support

5. **Configuration:**
   - Save user preferences
   - Custom operation profiles
   - Logging to file

6. **GUI Version:**
   - Cross-platform GUI
   - Visual progress tracking
   - Device management interface

### Architecture Considerations

- Keep platform abstraction clean
- Maintain safety features in all new features
- Preserve backward compatibility
- Document all public APIs
- Add tests for new functionality

## Testing Strategy

### Unit Tests

- Test utility functions (format_bytes, etc.)
- Test data structure methods
- Mock platform-specific calls

### Integration Tests

- Test device detection (requires real devices or mocks)
- Test operation flow (requires careful setup)
- Test error handling

### Manual Testing

- Test on real hardware (non-production devices!)
- Test on all supported platforms
- Test various device types
- Test error conditions

## Security Considerations

### Data Safety

- Multiple confirmation prompts
- Clear warnings about data loss
- Privilege requirements
- Mount detection

### Vulnerability Prevention

- No buffer overflows (Rust safety)
- Proper error handling
- No unsafe code in high-level modules
- Minimal unsafe code in platform modules

### Best Practices

- Run with least privilege when possible
- Validate all user input
- Check device paths carefully
- Prevent race conditions

## Performance

### Optimization Strategies

1. **Buffer Size:**
   - 1MB chunks for good performance
   - Balance between speed and memory

2. **Progress Updates:**
   - Update progress at reasonable intervals
   - Avoid excessive callback overhead

3. **Release Builds:**
   - Full optimization enabled
   - LTO (Link Time Optimization)
   - Strip debug symbols

### Benchmarks

Typical performance (will vary by hardware):
- HDD sequential write: 100-200 MB/s
- SSD sequential write: 400-500 MB/s
- NVMe sequential write: 1000-3000 MB/s

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on:
- Code style
- Pull request process
- Testing requirements
- Documentation standards

## License

MIT License - See LICENSE file for details.
