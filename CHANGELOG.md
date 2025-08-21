# Changelog

All notable changes to FuzzyTail will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2024-08-21

### 🎉 Initial Release

#### ✨ Added
- **Core tail functionality** with `tail` compatibility
  - `-n, --lines` - Number of lines to show
  - `-f, --follow` - Follow file changes 
  - `-q, --quiet` - Suppress file headers
  - `-v, --verbose` - Always show file headers
  - `-c, --bytes` - Show bytes instead of lines (framework)
  - Multiple file support with headers
  - Stdin input support

#### 🎨 Theming & Colors
- **6 beautiful themes** with true color support:
  - **Catppuccin** - Warm, pastel colors
  - **Dracula** - Dark theme with vibrant highlights
  - **Tokyo Night** - Modern dark theme
  - **Rose Pine** - Subtle, elegant colors  
  - **Lackluster** - Minimalist grayscale
  - **Miasma** - Earthy, muted tones
- **Smart syntax highlighting** for:
  - Timestamps (ISO, syslog formats)
  - IP addresses (IPv4, IPv6, MAC addresses)
  - Email addresses
  - URLs and hostnames
  - Log levels (ERROR, WARN, INFO, DEBUG, etc.)
  - Services (nginx, mysql, ssh, systemd, etc.)
  - HTTP status codes and methods
  - Authentication events
  - System processes and PIDs

#### 🔍 Advanced Filtering
- **Log level filtering** - `--level ERROR|WARN|INFO|DEBUG`
- **Regex include patterns** - `--include "pattern"`
- **Regex exclude patterns** - `--exclude "pattern"`
- **Combined filtering** - Mix and match all filter types
- **Smart log parsing** - Understands log structures automatically

#### 📊 Output Formats
- **Colorized text** - Beautiful default output
- **JSON format** - `--format json` for analysis tools
- **CSV format** - `--format csv` for spreadsheets
- **Structured data extraction** - Parses timestamps, IPs, services, etc.

#### ⚙️ Configuration
- **TOML configuration files**
  - User config: `~/.config/fuzzytail/config.toml`
  - System config: `/etc/fuzzytail/config.toml`
- **Theme management**
  - System themes: `/etc/fuzzytail/themes/`
  - User themes: `~/.config/fuzzytail/themes/`
- **Configurable buffering** - `--buffer-size` for performance tuning

#### 🚀 Performance & UX
- **Rust performance** - Zero-cost abstractions and memory safety
- **Configurable I/O buffering** - Optimized for large files
- **Enhanced defaults** - Helpful behavior when no files specified
- **Modern CLI** - Clean help text and argument parsing
- **Cross-platform** - Linux, macOS, Windows support

#### 🛠️ Developer Experience
- **Interactive mode framework** - `--interactive` (future feature)
- **Extensible architecture** - Easy to add themes and features
- **Comprehensive error handling** - Clear, helpful error messages
- **Hot configuration reload** - Change themes without restart

### 📋 Technical Details
- **Language**: Rust 2021 edition
- **Binary name**: `ft` (short and memorable)
- **Dependencies**: Minimal, carefully chosen crates
- **License**: MIT
- **Platform support**: Linux (primary), macOS, Windows

### 🎯 Compatibility
- **Drop-in `tail` replacement** - All standard arguments supported
- **Environment variables** - `FUZZYTAIL_THEME` support
- **Terminal compatibility** - Works with all modern terminals
- **Log rotation** - Proper handling of rotated log files

---

## 🚀 **What's Next?**

Planned features for future releases:

### 🎮 Interactive Mode
- Keyboard navigation (j/k, arrows)
- Live pause/resume (spacebar)
- Search within output
- Bookmarking and jumping

### 📈 Enhanced Analytics
- Real-time statistics
- Pattern frequency analysis
- Time-based filtering
- Log metrics dashboard

### 🔧 Extended Compatibility
- `--follow=name|descriptor` options
- `--retry` and `--pid` flags
- Size suffixes (K, M, G)
- Zero-terminated lines (`-z`)

### 🎨 Theme Enhancements
- RGB hex color support
- Theme hot-swapping
- Custom theme generator
- Theme sharing and import

### 📦 Distribution
- Package manager releases (brew, apt, dnf, etc.)
- Pre-built binaries for all platforms
- Docker images
- CI/CD automated releases

---

## 🤝 **Contributing**

See [CONTRIBUTING.md](CONTRIBUTING.md) for ways to get started.

## 📄 **License**

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.