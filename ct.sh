#!/usr/bin/env bash

# ct - Color Tail
# Usage: tail -f /var/log/logfile.log | ct
# Reads rules from /etc/colortail/ct.conf
# Supports base:COLOR to set default line color.
# Uses Perl regex with ANSI-skip to avoid recoloring colored spans.

set -uo pipefail

CONFIG_FILE="/etc/colortail/ct.conf"
declare -A LINE_COLORS
declare -A WORD_COLORS
BASE_COLOR="252"  # Dracula neutral gray; overridden by base: in config

# Read config
while IFS='' read -r configLine; do
  [[ "$configLine" =~ ^#|^$ ]] && continue
  if [[ "$configLine" =~ ^base:([0-9]{1,3})$ ]]; then
    BASE_COLOR="${BASH_REMATCH[1]}"
  elif [[ "$configLine" =~ ^line:(.*)=(.*)$ ]]; then
    LINE_COLORS["${BASH_REMATCH[1]}"]="${BASH_REMATCH[2]}"
  elif [[ "$configLine" =~ ^word:(.*)=(.*)$ ]]; then
    WORD_COLORS["${BASH_REMATCH[1]}"]="${BASH_REMATCH[2]}"
  fi
done < "$CONFIG_FILE"

# ANSI-skip for perl: skip already-colored regions
ANSI_SKIP='\x1b\[[0-9;]*m.*?\x1b\[0m(*SKIP)(*F)|'

while IFS='' read -r logLine; do
  coloredLine="$logLine"
  lineColored=0

  # Line-level color (first match wins)
  for pattern in "${!LINE_COLORS[@]}"; do
    if [[ "$logLine" =~ $pattern ]]; then
      ansiColor="${LINE_COLORS[$pattern]}"
      coloredLine=$'\033'"[38;5;${ansiColor}m${logLine}"$'\033'"[0m"
      lineColored=1
      break
    fi
  done

  # Word-level coloring only if no line-level rule
  if [[ "$lineColored" -eq 0 ]]; then
    for pattern in "${!WORD_COLORS[@]}"; do
      ansiColor="${WORD_COLORS[$pattern]}"
      # Return to BASE_COLOR after each match (not full reset)
      coloredLine="$(printf '%s' "$coloredLine" | perl -pe "s/${ANSI_SKIP}${pattern}/\e[38;5;${ansiColor}m\$&\e[38;5;${BASE_COLOR}m/g")"
    done
    # Apply base color to the whole line at the very end
    coloredLine=$'\033'"[38;5;${BASE_COLOR}m${coloredLine}"$'\033'"[0m"
  fi

  printf '%b\n' "$coloredLine"
done
