#!/usr/bin/env perl
# ct-fast — Color Tail (fast, single process, ordered rules)
# Usage: tail -f /var/log/logfile | ct-fast [--config /etc/colortail/ct.conf] [--base N]

use strict;
use warnings;
use Getopt::Long;

my $CONFIG = "/etc/colortail/ct.conf";
my $CLI_BASE = '';
GetOptions(
  "config|c=s" => \$CONFIG,
  "base|b=s"   => \$CLI_BASE,
) or die "bad args\n";

# ------------ read config (ordered) ------------
open my $fh, "<", $CONFIG or die "open $CONFIG: $!\n";
my $cfg_base = '';
my @line_rules;  # [qr/pat/, color]
my @word_rules;  # [qr/pat/, color]
while (my $l = <$fh>) {
  next if $l =~ /^\s*#/ || $l =~ /^\s*$/;
  chomp $l;
  if ($l =~ /^base:([0-9]{1,3})\s*$/) {
    $cfg_base = $1;
  } elsif ($l =~ /^line:(.*)=(\d{1,3})\s*$/) {
    my ($pat,$col) = ($1,$2);
    push @line_rules, [ qr/$pat/, $col ];
  } elsif ($l =~ /^word:(.*)=(\d{1,3})\s*$/) {
    my ($pat,$col) = ($1,$2);
    push @word_rules, [ qr/$pat/, $col ];
  }
}
close $fh;

# ------------ resolve BASE precedence ------------
# CLI --base > env CT_BASE_COLOR > env BASE_COLOR > config base:
my $ENV_BASE = $ENV{CT_BASE_COLOR} // $ENV{BASE_COLOR} // '';
my $BASE = $CLI_BASE ne '' ? $CLI_BASE
         : $ENV_BASE    ne '' ? $ENV_BASE
         : $cfg_base;

# Precompiled regex to skip existing ANSI colored spans
my $ANSI_SPAN = qr/\e\[[0-9;]*m.*?\e\[0m/;

# ------------ fast color helpers ------------
sub color_wrap {
  my ($text, $col) = @_;
  return "\e[38;5;${col}m$text\e[0m";
}
sub color_wrap_baseback {
  my ($text, $col, $base) = @_;
  # return to base (38;5;base) if set, else reset (39)
  my $reset = ($base ne '' ? "\e[38;5;${base}m" : "\e[39m");
  return "\e[38;5;${col}m$text$reset";
}

# ------------ stream ------------
binmode(STDIN,  ":unix");
binmode(STDOUT, ":unix");

LINE: while (defined(my $line = <STDIN>)) {
  chomp $line;

  # 1) line-level match (first wins)
  for my $r (@line_rules) {
    my ($re,$c) = @$r;
    if ($line =~ $re) {
      print color_wrap($line, $c), "\n";
      next LINE;
    }
  }

  my $out = $line;

  # 2) word-level coloring (ordered), skipping already-colored segments
  for my $r (@word_rules) {
    my ($re,$c) = @$r;
    # Replace uncolored matches only, restore BASE after each match
    $out =~ s{
      $ANSI_SPAN(*SKIP)(*F) | ($re)
    }{
      defined $1 ? color_wrap_baseback($1, $c, $BASE) : $&
    }egx;
  }

  # 3) apply BASE to the whole line (does not overpaint inner highlights)
  if ($BASE ne '') {
    # Wrap once, but avoid double-reset: insert base after any global reset
    # Simpler: just wrap entire output; existing spans will override as needed
    $out = "\e[38;5;${BASE}m$out\e[0m";
  }

  print $out, "\n";
}
