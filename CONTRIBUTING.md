# Contributing to FuzzyTail

Thank you for your interest in contributing to FuzzyTail! This document provides guidelines for contributing to the project.

## Development Setup

1. **Install Rust**: 
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Clone and build**:
   ```bash
   git clone https://github.com/your-username/fuzzytail
   cd fuzzytail
   cargo build
   ```

3. **Install themes for testing**:
   ```bash
   sudo mkdir -p /etc/fuzzytail/themes
   sudo cp themes/ft.conf.* /etc/fuzzytail/themes/
   ```

## Project Structure

```
fuzzytail/
├── src/
│   ├── main.rs          # CLI argument parsing and main entry point
│   ├── config.rs        # Configuration file handling
│   ├── theme.rs         # Theme parsing and color management
│   ├── colorizer.rs     # Text colorization logic
│   ├── filter.rs        # Line filtering (include/exclude/level)
│   ├── tail.rs          # Core tail functionality
│   ├── output.rs        # Output formatting (text/json/csv)
│   └── interactive.rs   # Interactive mode (future feature)
├── themes/              # Theme configuration files
└── README.md
```

## Adding New Features

### Adding a New Theme

1. Create a new theme file in `themes/ft.conf.mytheme`
2. Follow the existing format with base colors and word/line patterns
3. Test with: `ft --config-theme mytheme test.log`
4. Add documentation to `themes/README.md`

### Adding New Filters

1. Extend the `LineFilter` struct in `src/filter.rs`
2. Add new CLI arguments in `src/main.rs`
3. Update the `should_show_line` method
4. Add tests and documentation

### Adding New Output Formats

1. Add new format to `OutputFormat` enum in `src/output.rs`
2. Implement formatting logic in `OutputFormatter`
3. Add CLI option parsing
4. Test with various log formats

## Code Style

- Use `cargo fmt` to format code
- Run `cargo clippy` to check for warnings
- Follow Rust naming conventions
- Add documentation for public APIs
- Write tests for new functionality

## Testing

```bash
# Run tests
cargo test

# Test with sample data
echo "ERROR: Test message" | cargo run

# Test with real logs
cargo run -- /var/log/syslog

# Test all features
cargo run -- --level ERROR --format json --include "nginx" /var/log/nginx/access.log
```

## Performance Guidelines

- Use buffered I/O for file operations
- Compile regex patterns once and reuse
- Avoid unnecessary memory allocations in hot paths
- Test with large log files (> 100MB)

## Submitting Changes

1. **Fork** the repository
2. **Create** a feature branch: `git checkout -b feature-name`
3. **Make** your changes with tests
4. **Test** thoroughly with various log formats
5. **Commit** with descriptive messages
6. **Push** and create a Pull Request

## Pull Request Guidelines

- Describe the problem your PR solves
- Include examples of new functionality
- Add tests for new features
- Update documentation as needed
- Ensure CI passes

## Bug Reports

When reporting bugs, please include:

- FuzzyTail version: `ft --version`
- Operating system and version
- Minimal reproduction case
- Expected vs actual behavior
- Sample log data (if applicable)

## Feature Requests

We welcome feature requests! Please:

- Check existing issues first
- Describe the use case clearly
- Provide examples of desired behavior
- Consider implementation complexity

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and contribute
- Follow the project's technical standards

## Questions?

- Open an issue for questions
- Check existing documentation
- Look at similar implementations for guidance

Thank you for contributing to FuzzyTail! 🚀