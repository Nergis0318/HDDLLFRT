# Agent Guidelines for HDDLLFRT

This repository contains the source code for the HDD Low Level Format Tool (HDDLLFRT), a cross-platform utility written in Rust.

## 1. Build & Test Commands

### Build

- **Build (Dev):** `cargo build`
- **Build (Release):** `cargo build --release`
- **Check (Fast):** `cargo check`

### Test

- **Run All Tests:** `cargo test`
- **Run Single Test:** `cargo test -- <test_function_name>` (e.g., `cargo test -- test_device_detection`)
- **Run Tests with Logs:** `RUST_LOG=debug cargo test`

### Lint & Format

- **Format Code:** `cargo fmt` (Run this before committing)
- **Lint Code:** `cargo clippy` (Fix warnings where possible)

## 2. Code Style & Conventions

### General

- **Language:** Rust (2024 Edition).
- **Formatting:** Strictly adhere to `rustfmt` standards.
- **Organization:**
  - `src/main.rs`: CLI entry point and high-level command handlers.
  - `src/device/`: Core device abstractions and types.
  - `src/platform/`: OS-specific implementations (Windows, Linux, macOS).
  - `src/ui/`: CLI interaction and display logic.

### Naming

- **Functions/Variables:** `snake_case` (e.g., `detect_devices`, `user_input`).
- **Types (Structs/Enums):** `PascalCase` (e.g., `StorageDevice`, `DeviceType`).
- **Constants:** `SCREAMING_SNAKE_CASE` (e.g., `GENERIC_READ`).
- **Files:** `snake_case.rs`.

### Imports

Group imports in the following order:

1. Standard Library (`use std::...`)
2. External Crates (`use anyhow::...`, `use windows::...`)
3. Internal Modules (`use crate::device::...`)

### Error Handling

- **Library:** Use `anyhow` for application-level error handling.
- **Return Type:** Use `anyhow::Result<T>` for functions that can fail.
- **Context:** Always attach context to errors when propagating:

  ```rust
  .context("Failed to detect devices")?
  ```

- **Panic:** Avoid `unwrap()` or `expect()` in production code unless you are 100% certain it cannot fail. Use `?` propagation.

### OS-Specific Code

- **Feature Flags:** Use `#[cfg(target_os = "...")]` to guard platform-specific code.
- **Safety:** Minimize `unsafe` blocks. When using FFI (e.g., Windows API), wrap `unsafe` blocks tightly and justify if complex.
- **Windows API:** Use the `windows` crate (specifically `windows::Win32`).

### Logging & Output

- **User Output:** Use `println!` and `eprintln!` for CLI interaction (menus, prompts). Use `console::style` for coloring.
- **Debug/Info:** Use `log::info!`, `log::debug!`, etc., for internal diagnostics. Initialize `env_logger` in `main`.

## 3. Cursor/Copilot Rules

_(No specific existing rules found in .cursor/rules/ or .github/copilot-instructions.md. Follow standard Rust best practices defined above.)_

## 4. Agent Workflow

1. **Analyze:** specific platform implementation files (e.g., `src/platform/windows.rs`) before making OS-specific changes.
2. **Verify:** Always run `cargo check` after edits.
3. **Safety:** When modifying device operations (formatting/erasing), ensure safety checks (admin privileges, confirmation prompts) are preserved.
