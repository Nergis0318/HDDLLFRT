# HDDLLFRT - HDD Low Level Format Tool (Rust Edition)

A cross-platform storage device low-level format tool written in Rust, inspired by HDDGURU's HDD LLF Low Level Format Tool.

## Features

🔧 **Core Operations:**
- **Low Level Format**: Complete zero-fill of entire device
- **Quick Format**: Fast formatting of beginning and end sectors
- **Secure Erase**: Multiple-pass overwrite with different patterns
- **Device Verification**: Read and verify all sectors
- **S.M.A.R.T. Data**: View device health information

🖥️ **Cross-Platform Support:**
- ✅ Linux (tested on Ubuntu, Fedora, Arch)
- ✅ Windows (Windows 10/11)
- ✅ macOS (10.15+)

⚡ **Features:**
- Interactive CLI menu system
- Real-time progress tracking
- Multiple safety confirmations
- Automatic mount detection
- Administrator privilege checking
- Comprehensive error handling

## Installation

### Prerequisites

- Rust 1.70 or higher
- Administrator/root privileges for device operations

### Building from Source

```bash
# Clone the repository
git clone https://github.com/DevNergis/HDDLLFRT.git
cd HDDLLFRT

# Build release version
cargo build --release

# The binary will be in target/release/hddllfrt (or hddllfrt.exe on Windows)
```

### Quick Build Commands

```bash
# Debug build (faster compilation, slower execution)
cargo build

# Release build (optimized for performance)
cargo build --release

# Run directly without building separately
cargo run --release
```

## Usage

### Running the Tool

**Linux / macOS:**
```bash
sudo ./target/release/hddllfrt
```

**Windows:**
```cmd
# Run as Administrator
.\target\release\hddllfrt.exe
```

### Interactive Menu

The tool provides an interactive menu with the following options:

1. **List Devices** - Display all detected storage devices
2. **Low Level Format** - Perform complete zero-fill format
3. **Quick Format** - Fast format (beginning and end only)
4. **Secure Erase** - Multiple-pass secure erase (1-10 passes)
5. **Verify Device** - Read and verify all sectors
6. **View S.M.A.R.T. Data** - Display device health information
7. **Exit** - Quit the application

### Safety Features

The tool includes multiple safety mechanisms:

- ⚠️ **Administrator Check**: Requires elevated privileges
- ⚠️ **Mount Detection**: Prevents formatting mounted devices
- ⚠️ **Multiple Confirmations**: Two-step confirmation for destructive operations
- ⚠️ **Clear Warnings**: Prominent warnings about data loss

## Warning

⚠️ **CRITICAL WARNING** ⚠️

This tool performs **DESTRUCTIVE** operations on storage devices. When you format a device:

- **ALL DATA WILL BE PERMANENTLY ERASED**
- **THIS CANNOT BE UNDONE**
- **THERE IS NO DATA RECOVERY POSSIBLE**

Always verify you have selected the correct device before proceeding!

## Technical Details

### Device Detection

- **Linux**: Uses `/sys/block` and `/dev` to enumerate devices
- **Windows**: Uses Windows API to access Physical Drives
- **macOS**: Uses `diskutil` and IOKit for device enumeration

### Low Level Format Process

The low-level format operation:
1. Opens the device with exclusive write access
2. Writes zeros to the entire device in 1MB chunks
3. Tracks progress and provides real-time feedback
4. Syncs all data to ensure writes are committed
5. Verifies operation completion

### Secure Erase

Secure erase uses multiple passes with different patterns:
- Pass 1: 0x00 (all zeros)
- Pass 2: 0xFF (all ones)
- Pass 3: 0xAA (alternating pattern)
- Pass 4: 0x55 (inverse alternating pattern)

Patterns repeat for additional passes.

## Development

### Project Structure

```
HDDLLFRT/
├── src/
│   ├── main.rs              # Application entry point
│   ├── device/
│   │   ├── mod.rs           # Device data structures
│   │   └── operations.rs    # Format/erase operations
│   ├── platform/
│   │   ├── mod.rs           # Platform abstraction
│   │   ├── linux.rs         # Linux implementation
│   │   ├── windows.rs       # Windows implementation
│   │   └── macos.rs         # macOS implementation
│   └── ui/
│       └── mod.rs           # User interface
├── Cargo.toml               # Dependencies and metadata
└── README.md               # This file
```

### Running Tests

```bash
cargo test
```

### Code Style

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy
```

## Dependencies

Key dependencies:
- `clap` - Command line argument parsing
- `dialoguer` - Interactive CLI prompts
- `indicatif` - Progress bars
- `console` - Terminal styling
- `anyhow` - Error handling
- `sysinfo` - System information
- Platform-specific: `nix` (Linux), `windows` (Windows), `core-foundation` & `io-kit-sys` (macOS)

## Limitations

- S.M.A.R.T. data reading is basic (would need ATA command implementation for full support)
- Some USB devices may not be detected correctly
- NVMe-specific features are not fully implemented
- Requires elevated privileges for all operations

## Future Enhancements

- [ ] Full S.M.A.R.T. attribute reading via ATA commands
- [ ] Bad sector remapping
- [ ] Benchmark/speed testing
- [ ] Support for more device types
- [ ] GUI version
- [ ] Configuration file support
- [ ] Logging to file
- [ ] Resume interrupted operations

## License

MIT License - See LICENSE file for details

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues.

## Disclaimer

This software is provided "as is" without warranty of any kind. The authors are not responsible for any data loss or damage caused by the use of this software. Always backup your data before performing any disk operations.

## Credits

Inspired by HDDGURU's HDD LLF Low Level Format Tool.
Developed by DevNergis using Rust.
