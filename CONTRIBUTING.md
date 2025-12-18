# Contributing to HDDLLFRT

Thank you for your interest in contributing to HDDLLFRT! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/HDDLLFRT.git`
3. Create a new branch: `git checkout -b feature/your-feature-name`
4. Make your changes
5. Test your changes thoroughly
6. Commit your changes: `git commit -am 'Add some feature'`
7. Push to the branch: `git push origin feature/your-feature-name`
8. Create a Pull Request

## Development Setup

### Prerequisites

- Rust 1.70 or higher
- Cargo (comes with Rust)
- Git

### Building

```bash
# Clone the repository
git clone https://github.com/DevNergis/HDDLLFRT.git
cd HDDLLFRT

# Build debug version
cargo build

# Build release version
cargo build --release

# Run tests
cargo test
```

## Code Style

We follow the standard Rust style guidelines:

- Use `cargo fmt` to format your code
- Use `cargo clippy` to check for common mistakes
- Write clear, descriptive commit messages
- Comment complex logic

### Running Formatters and Linters

```bash
# Format code
cargo fmt

# Check for issues
cargo clippy -- -D warnings

# Check formatting without making changes
cargo fmt -- --check
```

## Testing

- Write tests for new functionality
- Ensure all existing tests pass
- Test on multiple platforms if possible (Linux, Windows, macOS)

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Platform-Specific Development

### Linux Development

- Test with various block device types (HDD, SSD, NVMe, USB)
- Ensure proper handling of `/sys/block` and `/dev` access
- Test with and without root privileges

### Windows Development

- Test with Physical Drive enumeration
- Ensure administrator privilege checks work correctly
- Test with various drive types

### macOS Development

- Test with `diskutil` integration
- Ensure proper IOKit integration
- Test with various disk types

## Pull Request Guidelines

1. **Keep PRs focused**: One feature or fix per PR
2. **Write clear descriptions**: Explain what your PR does and why
3. **Update documentation**: Update README.md if you add features
4. **Test thoroughly**: Ensure your changes work on at least one platform
5. **Follow code style**: Use `cargo fmt` and `cargo clippy`
6. **Add tests**: Add tests for new functionality

## Areas for Contribution

Here are some areas where contributions are particularly welcome:

### High Priority

- [ ] Full S.M.A.R.T. attribute reading implementation
- [ ] Better NVMe device detection and support
- [ ] Improved error handling and recovery
- [ ] Unit tests for core functionality
- [ ] Integration tests for device operations

### Medium Priority

- [ ] Speed/benchmark testing functionality
- [ ] Bad sector detection and remapping
- [ ] Support for more device types
- [ ] Configuration file support
- [ ] Logging to file

### Low Priority

- [ ] GUI version (using a framework like egui or iced)
- [ ] Support for more disk operations
- [ ] Internationalization (i18n)
- [ ] Plugin system

## Security Considerations

When contributing to this project, please keep in mind:

1. This tool performs destructive operations on storage devices
2. Always include multiple safety checks
3. Never reduce the number of confirmation prompts
4. Ensure proper privilege checking
5. Test thoroughly on non-production devices
6. Report security issues privately to the maintainers

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow the project's coding standards

## Questions?

If you have questions or need help:

1. Check the README.md for documentation
2. Look at existing code for examples
3. Open an issue for discussion
4. Reach out to the maintainers

## License

By contributing to HDDLLFRT, you agree that your contributions will be licensed under the MIT License.

Thank you for contributing! 🎉
