# 🚀 FuzzyTail (`ft`)

<div align="center">

**A modern, blazingly fast, and beautifully colorized replacement for `tail`**

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](https://opensource.org/licenses/MIT)
[![Build Status](https://img.shields.io/github/workflow/status/your-username/fuzzytail/CI?style=for-the-badge)](https://github.com/yodabytz/fuzzytail/actions)

*Transform your log viewing experience with intelligent syntax highlighting, powerful filtering, and multiple output formats*

![FuzzyTail Demo](https://via.placeholder.com/800x400/1a1b26/7aa2f7?text=FuzzyTail+Demo+%E2%80%A2+Beautiful+Colored+Logs)

</div>

---

## ✨ **Why FuzzyTail?**

While `tail` shows you raw text, **FuzzyTail transforms your logs into beautiful, readable, and actionable information**:

- 🎨 **6 gorgeous themes** with true color support
- 🔍 **Smart filtering** by log level, regex patterns, and more  
- 📊 **Multiple output formats**: colorized text, JSON, CSV
- ⚡ **Blazingly fast** Rust performance
- 🛠️ **Drop-in replacement** for standard `tail`
- 📱 **Modern UX** with helpful defaults and suggestions

---

## 🎯 **Quick Start**

```bash
# Install FuzzyTail
curl -sSL https://raw.githubusercontent.com/yodabytz/fuzzytail/main/install.sh | bash

# Use like tail, but better!
ft /var/log/syslog                    # Beautiful colored output
ft -f application.log                 # Follow mode with colors
ft --level ERROR /var/log/auth.log    # Show only errors
ft --format json app.log | jq         # JSON output for analysis
```

---

## 🚀 **Features**

### 🎨 **Beautiful Themes**
<table>
<tr>
<td align="center"><strong>Catppuccin</strong><br><img src="https://via.placeholder.com/120x80/fab387/1e1e2e?text=Catppuccin" alt="Catppuccin theme"></td>
<td align="center"><strong>Dracula</strong><br><img src="https://via.placeholder.com/120x80/bd93f9/282a36?text=Dracula" alt="Dracula theme"></td>
<td align="center"><strong>Tokyo Night</strong><br><img src="https://via.placeholder.com/120x80/7aa2f7/1a1b26?text=Tokyo+Night" alt="Tokyo Night theme"></td>
</tr>
<tr>
<td align="center"><strong>Rose Pine</strong><br><img src="https://via.placeholder.com/120x80/ebbcba/191724?text=Rose+Pine" alt="Rose Pine theme"></td>
<td align="center"><strong>Lackluster</strong><br><img src="https://via.placeholder.com/120x80/a8a8a8/1c1c1c?text=Lackluster" alt="Lackluster theme"></td>
<td align="center"><strong>Miasma</strong><br><img src="https://via.placeholder.com/120x80/c9a96e/1c1b19?text=Miasma" alt="Miasma theme"></td>
</tr>
</table>

### 🔍 **Advanced Filtering**
```bash
ft --level ERROR app.log              # Show ERROR level and above
ft --include "database|mysql" app.log # Include only matching patterns  
ft --exclude "DEBUG|TRACE" app.log    # Exclude debug information
ft --level WARN --include "user" app.log  # Combine multiple filters
```

### 📊 **Multiple Output Formats**
```bash
ft --format json app.log | jq '.level'     # JSON for analysis tools
ft --format csv app.log > logs.csv         # CSV for spreadsheets
ft app.log                                 # Beautiful colored text (default)
```

### ⚡ **Performance & Compatibility**
- **100% `tail` compatible** - drop-in replacement
- **Configurable buffering** for optimal performance
- **Memory efficient** - handles massive log files
- **Cross-platform** - works on Linux, macOS, Windows

---

## 📦 **Installation**

### 🚀 **Quick Install (Recommended)**

```bash
curl -sSL https://raw.githubusercontent.com/yodabytz/fuzzytail/main/install.sh | bash
```

This script will:
- ✅ Install Rust (if needed)
- ✅ Download and compile FuzzyTail
- ✅ Install the `ft` binary to `/usr/local/bin`
- ✅ Set up themes in `/etc/fuzzytail/themes`
- ✅ Create default configuration

### 🔧 **Manual Installation**

<details>
<summary>Click to expand manual installation steps</summary>

#### 1. Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 2. Build FuzzyTail
```bash
git clone https://github.com/yodabytz/fuzzytail
cd fuzzytail
cargo build --release
```

#### 3. Install Binary
```bash
sudo cp target/release/ft /usr/local/bin/ft
# OR for local user installation:
cp target/release/ft ~/.local/bin/ft
```

#### 4. Install Themes
```bash
sudo mkdir -p /etc/fuzzytail/themes
sudo cp themes/ft.conf.* /etc/fuzzytail/themes/
```

</details>

### 🍺 **Package Managers**

<details>
<summary>Package manager installations (coming soon!)</summary>

```bash
# Homebrew (macOS/Linux)
brew install fuzzytail

# Arch Linux
yay -S fuzzytail

# Debian/Ubuntu
sudo apt install fuzzytail

# Fedora
sudo dnf install fuzzytail
```

</details>

---

## 🎨 **Usage**

### **Basic Commands**
```bash
# View last 10 lines with beautiful colors
ft application.log

# Follow file changes (like tail -f)
ft -f application.log

# Show last 50 lines
ft -n 50 application.log

# Multiple files with headers
ft web.log app.log error.log

# Quiet mode (no headers)
ft -q *.log

# Read from stdin
echo "ERROR: Something failed" | ft
tail -f /var/log/syslog | ft
```

### **Advanced Filtering**
```bash
# Log level filtering (ERROR, WARN, INFO, DEBUG)
ft --level ERROR application.log

# Include only lines matching regex
ft --include "nginx|apache|mysql" /var/log/syslog

# Exclude lines matching regex  
ft --exclude "GET.*200|POST.*201" access.log

# Combine filters for precision
ft --level WARN --exclude "timeout" --include "user" app.log
```

### **Output Formats**
```bash
# JSON output for log analysis
ft --format json application.log

# Pipe to jq for processing
ft --format json app.log | jq '.[] | select(.level=="ERROR")'

# CSV export for spreadsheets
ft --format csv --level WARN app.log > warnings.csv

# Default beautiful colored output
ft application.log
```

### **Performance Tuning**
```bash
# Increase buffer size for large files
ft --buffer-size 131072 huge-logfile.log

# Disable colors for maximum speed
ft --no-color application.log
```

---

## ⚙️ **Configuration**

FuzzyTail uses TOML configuration files for themes and settings.

### **Configuration Locations**
- **User config**: `~/.config/fuzzytail/config.toml`
- **System config**: `/etc/fuzzytail/config.toml`

### **Example Configuration**
```toml
[general]
theme = "catppuccin"
buffer_size = 65536
follow_retry_interval = 1000

[themes]
builtin_path = "/etc/fuzzytail/themes"
user_path = "~/.config/fuzzytail/themes"
```

### **Theme Selection**
Change themes by editing your config file:
```toml
[general]
theme = "dracula"          # Dark theme with vibrant highlights
# theme = "tokyo-night"    # Modern dark theme  
# theme = "rose-pine"      # Subtle, elegant colors
# theme = "catppuccin"     # Warm, pastel colors
# theme = "lackluster"     # Minimalist grayscale
# theme = "miasma"         # Earthy, muted tones
```

---

## 🎯 **Examples & Use Cases**

### **System Administration**
```bash
# Monitor system logs in real-time
ft -f /var/log/syslog

# Find authentication failures
ft --include "failed|failure|denied" /var/log/auth.log

# Monitor only critical issues
ft --level ERROR -f /var/log/syslog

# Export security events for analysis
ft --format json --include "sudo|ssh|login" /var/log/auth.log > security.json
```

### **Application Development**
```bash
# Monitor application errors
ft --level ERROR -f application.log

# Track specific user activity
ft --include "user_id:12345" application.log

# Debug specific components
ft --include "database|cache|queue" --exclude "DEBUG" app.log

# Performance monitoring
ft --include "slow|timeout|performance" -f app.log
```

### **Web Server Analysis**
```bash
# Monitor nginx errors only
ft --include "nginx" --exclude "200|301|304" /var/log/nginx/access.log

# Export 4xx/5xx errors to CSV
ft --format csv --include "[45][0-9]{2}" access.log > errors.csv

# Real-time traffic monitoring
ft -f access.log | grep -E "(POST|PUT|DELETE)"
```

### **Log Analysis Workflows**
```bash
# Quick error overview
ft --level ERROR --format json app.log | \
  jq -r '.message' | sort | uniq -c | sort -nr

# Find patterns in large logs
ft --include "Exception|Error|Fatal" --format json huge.log | \
  jq -r '.timestamp + " " + .message' | head -20

# Export filtered data for external tools
ft --level WARN --format csv app.log | \
  awk -F',' '{print $1,$4}' > analysis.txt
```

---

## 🆚 **FuzzyTail vs Standard `tail`**

<table>
<tr>
<th>Feature</th>
<th>Standard <code>tail</code></th>
<th>FuzzyTail (<code>ft</code>)</th>
</tr>
<tr>
<td><strong>Basic file viewing</strong></td>
<td>✅ Plain text</td>
<td>✅ Beautiful syntax highlighting</td>
</tr>
<tr>
<td><strong>Follow mode (-f)</strong></td>
<td>✅ Basic following</td>
<td>✅ Enhanced following with colors</td>
</tr>
<tr>
<td><strong>Multiple files</strong></td>
<td>✅ With headers</td>
<td>✅ With colored headers</td>
</tr>
<tr>
<td><strong>Filtering</strong></td>
<td>❌ None</td>
<td>✅ Regex, log levels, advanced patterns</td>
</tr>
<tr>
<td><strong>Output formats</strong></td>
<td>❌ Text only</td>
<td>✅ Text, JSON, CSV</td>
</tr>
<tr>
<td><strong>Themes</strong></td>
<td>❌ None</td>
<td>✅ 6 beautiful themes + custom</td>
</tr>
<tr>
<td><strong>Smart parsing</strong></td>
<td>❌ None</td>
<td>✅ Timestamps, IPs, services, levels</td>
</tr>
<tr>
<td><strong>Performance</strong></td>
<td>⚡ C speed</td>
<td>⚡ Rust speed + optimizations</td>
</tr>
<tr>
<td><strong>Configuration</strong></td>
<td>❌ Command-line only</td>
<td>✅ Config files + CLI</td>
</tr>
<tr>
<td><strong>User Experience</strong></td>
<td>📝 Basic</td>
<td>🎨 Modern, helpful, intuitive</td>
</tr>
</table>

---

## 🛠️ **Development**

### **Building from Source**
```bash
git clone https://github.com/yodabytz/fuzzytail
cd fuzzytail
cargo build --release
```

### **Running Tests**
```bash
cargo test
```

### **Contributing**
We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### **Creating Custom Themes**
```bash
# Copy an existing theme
cp themes/ft.conf.catppuccin themes/ft.conf.mytheme

# Edit colors and patterns
vim themes/ft.conf.mytheme

# Test your theme
ft --config-theme mytheme test.log
```

---

## 📚 **Documentation**

- 📖 **[User Guide](docs/user-guide.md)** - Complete usage documentation
- 🎨 **[Theme Guide](themes/README.md)** - Creating and customizing themes  
- 🔧 **[Configuration Reference](docs/config.md)** - All configuration options
- 🤝 **[Contributing Guide](CONTRIBUTING.md)** - How to contribute
- 📋 **[Changelog](CHANGELOG.md)** - Version history and updates

---

## ❓ **FAQ**

<details>
<summary><strong>Q: How do I switch between themes?</strong></summary>

Edit `~/.config/fuzzytail/config.toml`:
```toml
[general]
theme = "dracula"  # Change this line
```
</details>

<details>
<summary><strong>Q: Can I use FuzzyTail as a drop-in replacement for tail?</strong></summary>

Yes! FuzzyTail supports all standard `tail` arguments:
- `-n, --lines` - Number of lines
- `-f, --follow` - Follow file changes  
- `-q, --quiet` - Suppress headers
- `-v, --verbose` - Always show headers
- `-c, --bytes` - Show bytes instead of lines

</details>

<details>
<summary><strong>Q: How do I disable colors for scripts?</strong></summary>

Use the `--no-color` flag:
```bash
ft --no-color logfile.log
```
</details>

<details>
<summary><strong>Q: Does FuzzyTail work with log rotation?</strong></summary>

Yes! FuzzyTail uses the same file watching mechanisms as standard `tail` and handles log rotation properly.
</details>

<details>
<summary><strong>Q: How do I create custom regex patterns?</strong></summary>

Edit your theme file and add patterns like:
```ini
word:MyApp\[ERROR\]=196
word:user_id:\d+=120  
word:192\.168\..*=117
```
</details>

---

## 🐛 **Troubleshooting**

### **Common Issues**

**Theme not loading:**
```bash
# Check theme file exists
ls /etc/fuzzytail/themes/

# Check config file
cat ~/.config/fuzzytail/config.toml
```

**Permission denied:**
```bash
# Install to user directory instead
cp target/release/ft ~/.local/bin/ft
```

**Colors not showing:**
```bash
# Check terminal color support
echo $TERM

# Force color output
export FORCE_COLOR=1
```

---

## 📄 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 **Acknowledgments**

- 🦀 **Rust Community** - For amazing crates and tooling
- 🎨 **Theme Creators** - Catppuccin, Dracula, Tokyo Night communities
- 📊 **Original colortail** - For inspiration
- 💝 **Contributors** - Everyone who makes FuzzyTail better

---

<div align="center">

**⭐ Star this repo if FuzzyTail makes your logs beautiful! ⭐**

[Report Bug](https://github.com/yodabytz/fuzzytail/issues) • [Request Feature](https://github.com/yodabytz/fuzzytail/issues) • [Contribute](CONTRIBUTING.md)

</div>