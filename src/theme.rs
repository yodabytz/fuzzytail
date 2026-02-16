use anyhow::{Context, Result, anyhow};
use regex::Regex;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Theme {
    pub name: String,
    pub base_color: Option<u8>,
    pub statusbar_bg: Option<Color>,
    pub statusbar_fg: Option<Color>,
    pub line_rules: Vec<ColorRule>,
    pub word_rules: Vec<ColorRule>,
}

#[derive(Debug, Clone)]
pub struct ColorRule {
    pub pattern: Regex,
    pub color: Color,
    pub original_pattern: String,
}

#[derive(Debug, Clone)]
pub enum Color {
    Xterm256(u8),
    TrueColor { r: u8, g: u8, b: u8 },
}

impl Color {
    pub fn to_ansi_fg(&self) -> String {
        match self {
            Color::Xterm256(n) => format!("\x1b[38;5;{}m", n),
            Color::TrueColor { r, g, b } => format!("\x1b[38;2;{};{};{}m", r, g, b),
        }
    }
    
    pub fn to_ansi_reset() -> &'static str {
        "\x1b[0m"
    }
}

impl Theme {
    const BUILTIN_CATPPUCCIN: &'static str = include_str!("../themes/ft.conf.catppuccin");
    const BUILTIN_DRACULA: &'static str = include_str!("../themes/ft.conf.dracula");
    const BUILTIN_LACKLUSTER: &'static str = include_str!("../themes/ft.conf.lackluster");
    const BUILTIN_MIASMA: &'static str = include_str!("../themes/ft.conf.miasma");
    const BUILTIN_ROSE_PINE: &'static str = include_str!("../themes/ft.conf.rose-pine");
    const BUILTIN_TOKYO_NIGHT: &'static str = include_str!("../themes/ft.conf.tokyo-night");

    pub fn load_from_file<P: AsRef<Path>>(path: P, name: String) -> Result<Self> {
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read theme file: {:?}", path.as_ref()))?;

        Self::parse_theme_contents(contents, name)
    }

    pub fn load_builtin(name: &str) -> Option<Result<Self>> {
        let contents = match name {
            "catppuccin" => Self::BUILTIN_CATPPUCCIN,
            "dracula" => Self::BUILTIN_DRACULA,
            "lackluster" => Self::BUILTIN_LACKLUSTER,
            "miasma" => Self::BUILTIN_MIASMA,
            "rose-pine" => Self::BUILTIN_ROSE_PINE,
            "tokyo-night" => Self::BUILTIN_TOKYO_NIGHT,
            _ => return None,
        };
        Some(Self::parse_theme_contents(contents.to_string(), name.to_string()))
    }
    
    fn parse_theme_contents(contents: String, name: String) -> Result<Self> {
        let mut base_color = None;
        let mut statusbar_bg = None;
        let mut statusbar_fg = None;
        let mut line_rules = Vec::new();
        let mut word_rules = Vec::new();

        for (line_num, line) in contents.lines().enumerate() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let line_context = || format!("Line {}: {}", line_num + 1, line);

            if let Some(caps) = Self::parse_base_line(line) {
                base_color = Some(caps);
            } else if line.starts_with("statusbar_bg:") {
                if let Ok(c) = Self::parse_color(line["statusbar_bg:".len()..].trim()) {
                    statusbar_bg = Some(c);
                }
            } else if line.starts_with("statusbar_fg:") {
                if let Ok(c) = Self::parse_color(line["statusbar_fg:".len()..].trim()) {
                    statusbar_fg = Some(c);
                }
            } else if let Some(rule) = Self::parse_line_rule(line).with_context(line_context)? {
                line_rules.push(rule);
            } else if let Some(rule) = Self::parse_word_rule(line).with_context(line_context)? {
                word_rules.push(rule);
            } else if !line.trim().is_empty() {
                eprintln!("Warning: Unrecognized line in theme {}: {}", name, line);
            }
        }

        Ok(Theme {
            name,
            base_color,
            statusbar_bg,
            statusbar_fg,
            line_rules,
            word_rules,
        })
    }
    
    fn parse_base_line(line: &str) -> Option<u8> {
        if line.starts_with("base:") {
            let color_str = &line[5..].trim();
            color_str.parse().ok()
        } else {
            None
        }
    }

    fn parse_u8_field(line: &str, prefix: &str) -> Option<u8> {
        if line.starts_with(prefix) {
            line[prefix.len()..].trim().parse().ok()
        } else {
            None
        }
    }
    
    fn parse_line_rule(line: &str) -> Result<Option<ColorRule>> {
        if line.starts_with("line:") {
            Self::parse_rule(&line[5..], "line")
        } else {
            Ok(None)
        }
    }
    
    fn parse_word_rule(line: &str) -> Result<Option<ColorRule>> {
        if line.starts_with("word:") {
            Self::parse_rule(&line[5..], "word")
        } else {
            Ok(None)
        }
    }
    
    fn parse_rule(rule_content: &str, rule_type: &str) -> Result<Option<ColorRule>> {
        if let Some(eq_pos) = rule_content.rfind('=') {
            let pattern_str = rule_content[..eq_pos].trim();
            let color_str = rule_content[eq_pos + 1..].trim();
            
            let pattern = Regex::new(pattern_str)
                .with_context(|| format!("Invalid regex pattern in {} rule: {}", rule_type, pattern_str))?;
            
            let color = Self::parse_color(color_str)
                .with_context(|| format!("Invalid color in {} rule: {}", rule_type, color_str))?;
            
            Ok(Some(ColorRule {
                pattern,
                color,
                original_pattern: pattern_str.to_string(),
            }))
        } else {
            Err(anyhow!("Invalid {} rule format, missing '=': {}", rule_type, rule_content))
        }
    }
    
    fn parse_color(color_str: &str) -> Result<Color> {
        if color_str.starts_with('#') {
            // RGB hex color: #ff5555
            if color_str.len() != 7 {
                return Err(anyhow!("Invalid hex color format: {}", color_str));
            }
            
            let r = u8::from_str_radix(&color_str[1..3], 16)?;
            let g = u8::from_str_radix(&color_str[3..5], 16)?;
            let b = u8::from_str_radix(&color_str[5..7], 16)?;
            
            Ok(Color::TrueColor { r, g, b })
        } else {
            // xterm-256 color: 123
            let color_num = color_str.parse::<u8>()
                .with_context(|| format!("Invalid color number: {}", color_str))?;
            
            if color_num > 255 {
                return Err(anyhow!("Color number out of range (0-255): {}", color_num));
            }
            
            Ok(Color::Xterm256(color_num))
        }
    }
    
    pub fn get_base_color_ansi(&self) -> String {
        if let Some(base) = self.base_color {
            format!("\x1b[38;5;{}m", base)
        } else {
            String::new()
        }
    }
}