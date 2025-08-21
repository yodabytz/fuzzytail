use crate::theme::{Theme, Color, ColorRule};
use regex::Regex;

pub struct Colorizer {
    theme: Theme,
    no_color: bool,
    ansi_span_regex: Regex,
}

impl Colorizer {
    pub fn new(theme: Theme, no_color: bool) -> Self {
        // Regex to match existing ANSI colored spans to avoid double-coloring
        let ansi_span_regex = Regex::new(r"\x1b\[[0-9;]*m.*?\x1b\[0m").unwrap();
        
        Self {
            theme,
            no_color,
            ansi_span_regex,
        }
    }
    
    pub fn colorize_line(&self, line: &str) -> String {
        if self.no_color {
            return line.to_string();
        }
        
        // 1. Check for line-level matches first (first match wins)
        for rule in &self.theme.line_rules {
            if rule.pattern.is_match(line) {
                return self.wrap_entire_line(line, &rule.color);
            }
        }
        
        // 2. Apply word-level coloring
        let mut result = line.to_string();
        
        for rule in &self.theme.word_rules {
            result = self.apply_word_rule(&result, rule);
        }
        
        // 3. Apply base color to the whole line (preserves inner highlights)
        if let Some(base_color) = self.theme.base_color {
            result = format!("\x1b[38;5;{}m{}\x1b[0m", base_color, result);
        }
        
        result
    }
    
    fn wrap_entire_line(&self, line: &str, color: &Color) -> String {
        format!("{}{}{}", color.to_ansi_fg(), line, Color::to_ansi_reset())
    }
    
    fn apply_word_rule(&self, text: &str, rule: &ColorRule) -> String {
        // Replace matches while avoiding already-colored segments
        rule.pattern.replace_all(text, |caps: &regex::Captures| {
            let matched_text = caps.get(0).unwrap().as_str();
            
            // Check if this match is inside an existing ANSI sequence
            if self.is_inside_ansi_sequence(text, caps.get(0).unwrap().start()) {
                matched_text.to_string()
            } else {
                self.wrap_with_base_restore(matched_text, &rule.color)
            }
        }).to_string()
    }
    
    fn is_inside_ansi_sequence(&self, text: &str, pos: usize) -> bool {
        for ansi_match in self.ansi_span_regex.find_iter(text) {
            if pos >= ansi_match.start() && pos < ansi_match.end() {
                return true;
            }
        }
        false
    }
    
    fn wrap_with_base_restore(&self, text: &str, color: &Color) -> String {
        let reset = if let Some(base) = self.theme.base_color {
            format!("\x1b[38;5;{}m", base)
        } else {
            "\x1b[39m".to_string() // Reset to default
        };
        
        format!("{}{}{}", color.to_ansi_fg(), text, reset)
    }
    
    pub fn theme_name(&self) -> &str {
        &self.theme.name
    }
    
    // For testing and debugging
    pub fn get_theme(&self) -> &Theme {
        &self.theme
    }
}