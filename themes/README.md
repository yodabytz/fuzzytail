# FuzzyTail Themes

This directory contains theme configuration files for FuzzyTail.

## Installation

Copy these theme files to your system themes directory:

```bash
sudo mkdir -p /etc/fuzzytail/themes
sudo cp ft.conf.* /etc/fuzzytail/themes/
```

Or to your user themes directory:

```bash
mkdir -p ~/.config/fuzzytail/themes
cp ft.conf.* ~/.config/fuzzytail/themes/
```

## Available Themes

- **ft.conf.catppuccin** - Warm, pastel colors inspired by the Catppuccin color scheme
- **ft.conf.dracula** - Dark theme with vibrant highlights 
- **ft.conf.tokyo-night** - Modern dark theme with blue accents
- **ft.conf.rose-pine** - Subtle, elegant colors with rose tones
- **ft.conf.lackluster** - Minimalist grayscale theme
- **ft.conf.miasma** - Earthy, muted tones

## Usage

Set your preferred theme in `~/.config/fuzzytail/config.toml`:

```toml
[general]
theme = "catppuccin"
```

## Theme Format

Each theme file uses a simple format:

```ini
# Base color for non-matching text
base:146

# Line-level highlighting (first match wins)
line:ALERT=196

# Word-level highlighting (processed in order)
word:ERROR=203
word:WARN=214
word:INFO=146
word:DEBUG=102

# Pattern examples
word:(?:[0-9]{1,3}\.){3}[0-9]{1,3}=117        # IP addresses
word:[A-Za-z0-9._%+\-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}=218   # Email addresses
```

Colors can be:
- **xterm-256**: Numbers 0-255 (e.g., `203`)
- **RGB hex**: True color hex codes (e.g., `#ff5555`)