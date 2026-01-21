## HDDLLFRT Agent Guide

Purpose
- This repository is a cross-platform, destructive disk utility written in Rust.
- Safety and correctness are more important than speed of iteration.

Sources of truth
- Project overview and commands: `README.md`.
- Architecture details: `docs/ARCHITECTURE.md`.
- Usage and safety guidance: `docs/USAGE.md`.
- CI expectations: `.github/workflows/ci.yml`.
- AI coding instructions (must follow): `.github/copilot-instructions.md`.

Critical safety context
- The tool performs destructive operations (zero-fill, secure erase).
- Never weaken safety prompts or privilege checks.
- Do not change device-selection flow without adding guardrails.

Build, lint, and test
- Build (debug): `cargo build`.
- Build (release): `cargo build --release`.
- Run (release): `cargo run --release`.
- Format: `cargo fmt`.
- Format check (CI): `cargo fmt -- --check`.
- Lint (CI): `cargo clippy -- -D warnings -A dead_code -A clippy::upper_case_acronyms`.
- Tests: `cargo test`.
- Single test by name: `cargo test <test_name>`.
- Single test in module: `cargo test module_name::test_name`.
- Single integration test target: `cargo test --test <test_file>`.

CI parity notes
- CI runs format check, clippy with warnings denied, build, tests, and release build.
- Keep clippy clean; follow the allow list used in CI when needed.

Repository layout
- Entry point: `src/main.rs`.
- UI layer: `src/ui/mod.rs`.
- Device model and data: `src/device/mod.rs`.
- Device operations: `src/device/operations.rs`.
- Platform abstraction: `src/platform/mod.rs`.
- Platform-specific implementations: `src/platform/linux.rs`, `src/platform/windows.rs`, `src/platform/macos.rs`.

Architecture rules (from Copilot instructions)
- Use `#[cfg(target_os = "...")]` for OS-specific behavior.
- Add platform-specific logic in the OS file and expose via `src/platform/mod.rs`.
- Treat devices as raw files and open via `platform::open_device_exclusive`.
- Keep UI logic in `src/ui/`, core operations in `src/device/`.
- Keep `src/main.rs` lean and delegate to UI and device modules.

Error handling
- Use `anyhow::Result<T>` for app-level errors.
- Use `thiserror` for library error types when defining new error enums.
- Add context with `.context("...")` before propagating errors.
- Prefer early returns with `?` and minimal nesting.

Safety checks
- Always verify admin/root privileges before destructive operations.
- Always ensure device is unmounted before write operations.
- Maintain two-step confirmation for destructive actions.
- Log and surface user-facing errors clearly.

Code style
- Formatting: standard `rustfmt` defaults.
- Imports: group by std, external crates, then local modules; keep lines tidy.
- Types: use explicit types for public APIs and struct fields.
- Naming: `snake_case` for functions/vars, `PascalCase` for types/traits, `SCREAMING_SNAKE_CASE` for constants.
- Enums: prefer explicit variants (`Unknown`, `Warning`) instead of magic values.
- Prefer `Result<T>` return types; avoid panics in normal control flow.
- Use `const` for buffer sizes and magic numbers used across functions.

Device operation patterns
- Use buffered reads/writes with chunk sizes (4MB or 10MB) as seen in `src/device/operations.rs`.
- Always `seek` before writing, and `sync_all()` after writes.
- Progress updates go through callback functions or progress bars.

Platform-specific guidance
- Linux: use sysfs (`/sys/block`) and `/proc/mounts` for discovery and mount checks.
- Windows: use Win32 APIs via the `windows` crate; lock and dismount volumes carefully.
- macOS: use `diskutil` and parse output; keep it defensive and error-aware.

Logging
- Logging uses `log` + `env_logger` initialized in `main`.
- Prefer info-level logging for operational status and warnings for risky states.

Testing notes
- There are currently limited tests; add unit tests for pure logic (e.g., `format_bytes`).
- Avoid tests that perform real device operations unless fully mocked.

Documentation expectations
- Update `README.md` or `docs/USAGE.md` when adding user-visible features.
- Keep safety warnings prominent when behavior changes.

When editing existing code
- Preserve the CLI interaction style (dialoguer + console + indicatif).
- Preserve warning banners, confirmation flow, and error messages.
- Keep cross-platform dispatching centralized in `src/platform/mod.rs`.

Common commands for agents
- List devices: run the app in a privileged shell.
- Check formatting: `cargo fmt -- --check`.
- Run clippy locally before committing: `cargo clippy -- -D warnings -A dead_code -A clippy::upper_case_acronyms`.

Gotchas
- Running device operations without elevated privileges will fail.
- Be cautious with Unicode symbols in UI output; the code already uses them.
- This repo targets Rust 2024 edition; keep new code compatible.
