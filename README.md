# Color Tail

**Color Tail** is a small script that applies configurable color highlights to streaming log data. It reads from a configuration file (`/etc/colortail/ct.conf`) and colorizes matching lines or words using ANSI escape codes.

---

## Features

- **Line Matching**: Colorize an entire line if it matches a specific regex.
- **Word Matching**: Highlight only specific words or patterns in each line.
- **256-Color Support**: Use any of the extended ANSI colors (0–255).
- **Date, IP, and Email Matching**: Easily highlight timestamps, IP addresses, or email addresses.

---

## Requirements

- Bash 4.x or higher
- `sed` (supports extended regex via `-E`)

---

## Installation

1. **Clone or Download** this repo.
2. **Copy** the `ct.sh` script to a folder in your `$PATH` (e.g., `/usr/local/bin/ct`).
3. **Make it Executable**:
   ```bash
   chmod +x /usr/local/bin/ct
   ```

## Create configuration directory

```
sudo mkdir /etc/colortail
sudo cp ct.conf /etc/colortail/ct.conf
```

## Edit the config to your liking
```
vim /etc/colortail/ct.conf
```
## Usage

```
tail -f /var/log/logfile.log | ct
```
