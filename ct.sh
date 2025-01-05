#!/usr/bin/env bash

# ct - Color Tail
# Usage: tail -f /var/log/logfile.log | ct
# Reads rules from /etc/colortail/ct.conf
# Uses extended regex for reliable matching.
# Uses IFS='' and printf for reliable, unbroken line reads and output.

CONFIG_FILE="/etc/colortail/ct.conf"
declare -A LINE_COLORS
declare -A WORD_COLORS

while IFS='' read -r configLine; do
  [[ "$configLine" =~ ^#|^$ ]] && continue
  if [[ "$configLine" =~ ^line:(.*)=(.*)$ ]]; then
    LINE_COLORS["${BASH_REMATCH[1]}"]="${BASH_REMATCH[2]}"
  elif [[ "$configLine" =~ ^word:(.*)=(.*)$ ]]; then
    WORD_COLORS["${BASH_REMATCH[1]}"]="${BASH_REMATCH[2]}"
  fi
done < "$CONFIG_FILE"

while IFS='' read -r logLine; do
  coloredLine="$logLine"
  for pattern in "${!LINE_COLORS[@]}"; do
    if [[ "$logLine" =~ $pattern ]]; then
      ansiColor="${LINE_COLORS[$pattern]}"
      coloredLine="\033[38;5;${ansiColor}m${logLine}\033[0m"
      break
    fi
  done
  if [[ "$coloredLine" == "$logLine" ]]; then
    for pattern in "${!WORD_COLORS[@]}"; do
      ansiColor="${WORD_COLORS[$pattern]}"
      coloredLine="$(printf '%s' "$coloredLine" | sed -E "s/$pattern/\x1b[38;5;${ansiColor}m&\x1b[0m/g")"
    done
  fi
  printf '%b\n' "$coloredLine"
done
