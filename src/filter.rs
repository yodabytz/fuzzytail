use anyhow::{Context, Result};
use regex::Regex;

#[derive(Clone)]
pub struct LineFilter {
    include_regex: Option<Regex>,
    exclude_regex: Option<Regex>,
    level_filter: Option<LogLevel>,
}

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Emergency,
    Alert,
    Critical,
    Error,
    Warning,
    Notice,
    Info,
    Debug,
}

impl LogLevel {
    fn from_str(level: &str) -> Option<LogLevel> {
        match level.to_uppercase().as_str() {
            "EMERG" | "EMERGENCY" => Some(LogLevel::Emergency),
            "ALERT" => Some(LogLevel::Alert),
            "CRIT" | "CRITICAL" => Some(LogLevel::Critical),
            "ERR" | "ERROR" => Some(LogLevel::Error),
            "WARN" | "WARNING" => Some(LogLevel::Warning),
            "NOTICE" => Some(LogLevel::Notice),
            "INFO" => Some(LogLevel::Info),
            "DEBUG" | "TRACE" => Some(LogLevel::Debug),
            _ => None,
        }
    }

    fn priority(&self) -> u8 {
        match self {
            LogLevel::Emergency => 0,
            LogLevel::Alert => 1,
            LogLevel::Critical => 2,
            LogLevel::Error => 3,
            LogLevel::Warning => 4,
            LogLevel::Notice => 5,
            LogLevel::Info => 6,
            LogLevel::Debug => 7,
        }
    }
}

impl LineFilter {
    pub fn new(
        include: Option<String>,
        exclude: Option<String>,
        level: Option<String>,
    ) -> Result<Self> {
        let include_regex = if let Some(pattern) = include {
            Some(Regex::new(&pattern).context("Invalid include regex pattern")?)
        } else {
            None
        };

        let exclude_regex = if let Some(pattern) = exclude {
            Some(Regex::new(&pattern).context("Invalid exclude regex pattern")?)
        } else {
            None
        };

        let level_filter = if let Some(level_str) = level {
            LogLevel::from_str(&level_str)
                .with_context(|| format!("Invalid log level: {}", level_str))?
                .into()
        } else {
            None
        };

        Ok(Self {
            include_regex,
            exclude_regex,
            level_filter,
        })
    }

    pub fn should_show_line(&self, line: &str) -> bool {
        // Check exclude pattern first (most restrictive)
        if let Some(exclude_regex) = &self.exclude_regex {
            if exclude_regex.is_match(line) {
                return false;
            }
        }

        // Check include pattern
        if let Some(include_regex) = &self.include_regex {
            if !include_regex.is_match(line) {
                return false;
            }
        }

        // Check log level filter
        if let Some(target_level) = self.level_filter {
            if !self.line_matches_level(line, target_level) {
                return false;
            }
        }

        true
    }

    fn line_matches_level(&self, line: &str, target_level: LogLevel) -> bool {
        let detected_level = self.detect_log_level(line);
        
        if let Some(detected) = detected_level {
            // Show messages at target level or higher priority (lower number)
            detected.priority() <= target_level.priority()
        } else {
            // If no level detected, show for DEBUG and INFO levels only
            matches!(target_level, LogLevel::Debug | LogLevel::Info)
        }
    }

    fn detect_log_level(&self, line: &str) -> Option<LogLevel> {
        let line_upper = line.to_uppercase();
        
        // Check for common log level patterns
        if line_upper.contains("EMERG") || line_upper.contains("EMERGENCY") {
            Some(LogLevel::Emergency)
        } else if line_upper.contains("ALERT") {
            Some(LogLevel::Alert)
        } else if line_upper.contains("CRIT") || line_upper.contains("CRITICAL") {
            Some(LogLevel::Critical)
        } else if line_upper.contains("ERROR") || line_upper.contains("ERR") {
            Some(LogLevel::Error)
        } else if line_upper.contains("WARN") || line_upper.contains("WARNING") {
            Some(LogLevel::Warning)
        } else if line_upper.contains("NOTICE") {
            Some(LogLevel::Notice)
        } else if line_upper.contains("INFO") {
            Some(LogLevel::Info)
        } else if line_upper.contains("DEBUG") || line_upper.contains("TRACE") {
            Some(LogLevel::Debug)
        } else {
            None
        }
    }

    pub fn is_active(&self) -> bool {
        self.include_regex.is_some() || self.exclude_regex.is_some() || self.level_filter.is_some()
    }
}