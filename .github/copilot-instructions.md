# HDDLLFRT AI Coding Instructions

## Project Overview
HDDLLFRT is a cross-platform low-level format tool written in Rust. It performs destructive operations (zero-fill, secure erase) on storage devices. Safety and platform abstraction are paramount.

## Architecture & Core Patterns

### Platform Abstraction
- **Pattern:** The project uses compile-time conditional compilation (`#[cfg(target_os = "...")]`) to handle OS differences.
- **Location:** `src/platform/mod.rs` acts as the facade, dispatching calls to `linux.rs`, `windows.rs`, or `macos.rs`.
- **Rule:** When adding platform-specific functionality, implement it in the respective OS module and expose it via `src/platform/mod.rs`.

### Device Operations
- **Pattern:** Storage devices are treated as raw files.
- **Location:** `src/device/operations.rs` contains the core logic (`low_level_format`, `verify_device`).
- **Implementation:** Uses `std::fs::OpenOptions` with `write(true)` or `read(true)`.
- **Safety:** Operations must ensure the device is unmounted (where applicable) and the user has confirmed the action.

### User Interface
- **Pattern:** CLI interaction is separated from logic.
- **Location:** `src/ui/` handles menus (`dialoguer`), progress bars (`indicatif`), and output styling (`console`).
- **Rule:** Keep `main.rs` clean; delegate user interaction to `src/ui/` and business logic to `src/device/`.

## Critical Workflows

### Building & Running
- **Build:** `cargo build --release` (Release mode is critical for I/O performance).
- **Run:** The application **requires Administrator/Root privileges** to access raw devices.
  - Windows: Run terminal as Administrator.
  - Linux/macOS: Run with `sudo`.
- **Debugging:** `cargo run` works for UI testing, but actual device operations will fail without elevated permissions.

### Error Handling
- **Library:** Uses `anyhow` for application-level error handling and `thiserror` for library-level errors.
- **Pattern:** Use `.context("...")` to add context to errors before propagating them.

## Key Files
- `src/main.rs`: Entry point, main loop, and high-level error handling.
- `src/platform/mod.rs`: OS-specific dispatch logic.
- `src/device/operations.rs`: The "dangerous" code (writing to disk).
- `src/device/mod.rs`: Data structures (`StorageDevice`, `SmartData`).

## Safety Guidelines
1. **Destructive Actions:** Always ensure multiple confirmations before writing to a device.
2. **Device Filtering:** Be careful not to list or target the system drive unless explicitly intended (though the tool currently lists all).
3. **Buffer Sizes:** Use appropriate buffer sizes (e.g., 1MB - 10MB) for I/O operations to ensure performance.
