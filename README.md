# FuzzyTail (`ft`)

A modern, fast replacement for `tail` with split-pane log monitoring, syntax highlighting, and powerful filtering. Built in Rust.

---

## Features

- **Split-pane monitoring** - Tail multiple files simultaneously in a multitail-style split-screen layout with per-file status bars
- **Auto-follow** - Multiple files automatically enter follow mode with split panes
- **6 themes** - Tokyo Night, Catppuccin, Dracula, Rose Pine, Lackluster, Miasma
- **Smart syntax highlighting** - Timestamps, IPs, log levels, HTTP methods, services, and more are automatically colorized
- **Filtering** - Include/exclude patterns with regex, filter by log level
- **Multiple output formats** - Colorized text, JSON, CSV
- **Drop-in `tail` replacement** - All standard flags work: `-f`, `-n`, `-c`, `-q`, `-v`
- **Theme-controlled status bars** - Each theme defines its own status bar colors via `statusbar_bg` and `statusbar_fg`

---

## Quick Start

```bash
# Install
git clone https://github.com/yodabytz/fuzzytail
cd fuzzytail
cargo build --release
sudo cp target/release/ft /usr/local/bin/ft
sudo mkdir -p /etc/fuzzytail/themes
sudo cp themes/ft.conf.* /etc/fuzzytail/themes/
```

---

## Usage

### Single file
```bash
ft /var/log/syslog                        # Last 10 lines, colorized
ft -f /var/log/syslog                     # Follow mode
ft -n 50 /var/log/auth.log                # Last 50 lines
```

### Multi-pane monitoring
```bash
# Split-pane view with status bars (auto-follows)
ft /var/log/syslog /var/log/auth.log /var/log/mail.log

# Exclude noisy lines
ft --exclude "CRON" /var/log/syslog /var/log/auth.log

# Filter by log level across multiple files
ft --level ERROR /var/log/syslog /var/log/nginx/error.log
```

In multi-pane mode:
- Each file gets its own pane with a status bar showing the filename, line count, and timestamp
- Press `q` or `Esc` to quit
- Press `h` for help
- Press `1`-`9` to view a single file full-screen

### Filtering
```bash
ft --level ERROR app.log                  # Show ERROR and above
ft --include "nginx|mysql" /var/log/syslog  # Only matching lines
ft --exclude "GET.*200" access.log        # Exclude patterns
ft --level WARN --exclude "timeout" app.log  # Combine filters
```

### Output formats
```bash
ft --format json app.log                  # JSON output
ft --format csv app.log > logs.csv        # CSV export
ft --no-color app.log                     # Plain text
```

### Pipe support
```bash
journalctl -f | ft                        # Colorize any stream
cat app.log | ft --level ERROR            # Filter piped input
```

---

## Configuration

Config file: `~/.config/fuzzytail/config.toml`

```toml
[general]
theme = "tokyo-night"
buffer_size = 8192
follow_retry_interval = 1000

[themes]
builtin_path = "/etc/fuzzytail/themes"
user_path = "~/.config/fuzzytail/themes"
```

### Available themes

| Theme | Style |
|-------|-------|
| `tokyo-night` | Modern dark with blue-purple accents |
| `catppuccin` | Warm pastels on dark background |
| `dracula` | Classic dark with vibrant highlights |
| `rose-pine` | Subtle, elegant muted tones |
| `lackluster` | Minimalist monochrome |
| `miasma` | Earthy, warm browns and greens |

---

## Theme format

Themes are plain text files in `/etc/fuzzytail/themes/`. Format:

```ini
# Base text color (xterm-256)
base:147

# Status bar colors (xterm-256)
statusbar_bg:103
statusbar_fg:255

# Line highlight: entire line colored if pattern matches
line:ALERT=210

# Word highlight: matching text colored
word:ERROR=210
word:(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\s+\d+\s+\d+:\d+:\d+=137

# Colors are xterm-256 numbers (0-255)
```

Create custom themes by copying an existing one:
```bash
cp /etc/fuzzytail/themes/ft.conf.tokyo-night /etc/fuzzytail/themes/ft.conf.mytheme
```

---

## Command reference

```
ft [OPTIONS] [FILES...]

Options:
  -n, --lines <N>       Number of lines to show (default: 10)
  -c, --bytes <N>       Show last N bytes instead of lines
  -f, --follow          Follow file changes
  --no-follow           Disable auto-follow for multiple files
  -q, --quiet           Never show file headers
  -v, --verbose         Always show file headers
  --include <REGEX>     Show only lines matching pattern
  --exclude <REGEX>     Hide lines matching pattern
  --level <LEVEL>       Filter by log level (ERROR, WARN, INFO, DEBUG)
  --format <FMT>        Output format: text, json, csv
  --no-color            Disable colors
  --buffer-size <N>     Buffer size in bytes (default: 65536)
  --config <PATH>       Config file path
  -h, --help            Show help
```

---

## Building

```bash
# Requires Rust toolchain
cargo build --release

# Run tests
cargo test
```

---

## License

MIT
