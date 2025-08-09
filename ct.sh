#!/usr/bin/env bash
# ct - Color Tail (ordered, Perl regex, ANSI-safe, solid email color)

set -uo pipefail
export LC_ALL=C

CONFIG_FILE="/etc/colortail/ct.conf"

BASE_COLOR=""
LINE_PATTERNS=()
LINE_COLORS=()
WORD_PATTERNS=()
WORD_COLORS=()

# Read config IN ORDER
while IFS='' read -r configLine; do
  [[ "$configLine" =~ ^#|^$ ]] && continue
  if   [[ "$configLine" =~ ^base:([0-9]{1,3})$ ]]; then
    BASE_COLOR="${BASH_REMATCH[1]}"
  elif [[ "$configLine" =~ ^line:(.*)=(.*)$ ]]; then
    LINE_PATTERNS+=("${BASH_REMATCH[1]}")
    LINE_COLORS+=("${BASH_REMATCH[2]}")
  elif [[ "$configLine" =~ ^word:(.*)=(.*)$ ]]; then
    WORD_PATTERNS+=("${BASH_REMATCH[1]}")
    WORD_COLORS+=("${BASH_REMATCH[2]}")
  fi
done < "$CONFIG_FILE"

perl_color() {
  BASE="$1" COL="$2" PAT="$3" \
  perl -pe '
    use strict; use warnings;
    my $base = defined $ENV{BASE} ? $ENV{BASE} : "";
    my $col  = $ENV{COL};
    my $pat  = qr{$ENV{PAT}};                            # safe for / in patterns
    my $ansi = qr{\x1b\[[0-9;]*m.*?\x1b\[0m};            # existing colored spans
    s/$ansi(*SKIP)(*F)|$pat/
      "\x1b[38;5;".$col."m".$&.($base ne "" ? "\x1b[38;5;".$base."m" : "\x1b[0m]")
    /eg;
  '
}

while IFS='' read -r logLine; do
  coloredLine="$logLine"

  # Line-level (first match wins)
  for i in "${!LINE_PATTERNS[@]}"; do
    if [[ "$logLine" =~ ${LINE_PATTERNS[$i]} ]]; then
      coloredLine=$'\033[38;5;'"${LINE_COLORS[$i]}m${logLine}"$'\033[0m'
      printf '%b\n' "$coloredLine"
      continue 2
    fi
  done

  # Word-level (in order; emails before hostnames in config)
  for i in "${!WORD_PATTERNS[@]}"; do
    coloredLine="$(printf '%s' "$coloredLine" | perl_color "${BASE_COLOR}" "${WORD_COLORS[$i]}" "${WORD_PATTERNS[$i]}")"
  done

  # Apply base color last (does NOT block word colors)
  if [[ -n "$BASE_COLOR" ]]; then
    coloredLine=$'\033[38;5;'"${BASE_COLOR}m${coloredLine}"$'\033[0m'
  fi

  printf '%b\n' "$coloredLine"
done
