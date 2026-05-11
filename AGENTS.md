# Agent Guidelines for HDDLLFRT

Rust 2024 edition. Cross-platform HDD low-level format CLI tool.

## Build & Verify

```bash
cargo build              # dev build
cargo build --release    # optimized (LTO, strip, single codegen-unit)
cargo check              # fast type-check
cargo test               # unit tests only (in src/device/)
cargo fmt -- --check     # CI enforces formatting
cargo clippy -- -D warnings -A dead_code -A clippy::upper_case_acronyms  # CI rule
```

CI runs build + test on all three platforms (ubuntu, windows, macos). Platform code compiles conditionally — `cargo check` on one OS only verifies that platform's paths.

## Architecture

Single-crate binary (`hddllfrt`). No workspace, no integration tests, no examples.

```
src/
  main.rs          — CLI entry, menu loop, operation orchestration
  device/
    mod.rs         — StorageDevice struct, DeviceType enum, unit tests
    operations.rs  — format/erase/verify logic (low_level_format, quick_format, secure_erase, verify_device)
  platform/
    mod.rs         — DeviceHandle (File wrapper with OS-specific locks), cross-platform API
    windows.rs     — Win32 API via `windows` crate (CreateFileW, DeviceIoControl, FSCTL_LOCK_VOLUME)
    linux.rs       — /sys/block enumeration, flock-based exclusive open
    macos.rs       — diskutil plist parsing, flock-based exclusive open
  ui/
    mod.rs         — dialoguer menus, indicatif progress bars, console styling
```

`platform::open_device_exclusive` returns `DeviceHandle`. On Windows it locks and dismounts all volumes on the target disk before opening. On Unix it uses `flock(LOCK_EX | LOCK_NB)`.

## Key Dependencies

- `windows` crate (0.52) — Win32 FFI, feature-gated per API group
- `dialoguer` — interactive prompts (Select, Confirm, Input)
- `indicatif` — progress bars
- `console` — terminal styling
- `anyhow` — error handling, `.context()` on all fallible calls
- `plist` (macOS only) — parses diskutil output
- `nix` (Linux only) — ioctl/mount abstractions

## Conventions

- Error handling: `anyhow::Result<T>` everywhere, `.context("...")` on propagation, no bare `unwrap()` in production paths.
- Platform code gated with `#[cfg(target_os = "...")]`. The `platform/mod.rs` re-exports dispatch at runtime via cfg blocks, not traits.
- `DeviceHandle` in `platform/mod.rs` implements `Read + Write + Seek` + manual `sync_all` + Unix `write_at`/`read_at`. The Windows variant holds extra `_locks: Vec<File>` to keep volume locks alive.
- Admin privilege check: `net session` on Windows, `geteuid() == 0` on Unix. Non-admin is a soft warning on startup, hard requirement before destructive ops.
- Two-step confirmation before any destructive operation. Safety checks (admin + mount detection) in `ui::check_prerequisites`.
- Logging: `env_logger` at `Info` level by default. Use `RUST_LOG=debug` for verbose.

## What's NOT Here

- No tests outside `src/device/` (operations and platform code are untested).
- No CI lint for unused imports or dead code beyond clippy defaults (dead_code is explicitly allowed).
- No `CLAUDE.md`, `.cursor/rules/`, or `.github/copilot-instructions.md`.
- Release profile uses `panic = "abort"` and `strip = true` — no debug info in release builds.
