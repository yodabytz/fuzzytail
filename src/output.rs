use serde_json::{json, Value};
use std::collections::HashMap;
use regex::Regex;

pub enum OutputFormat {
    Text,
    Json,
    Csv,
}

impl OutputFormat {
    pub fn from_string(format: &str) -> OutputFormat {
        match format.to_lowercase().as_str() {
            "json" => OutputFormat::Json,
            "csv" => OutputFormat::Csv,
            _ => OutputFormat::Text,
        }
    }
}

pub struct OutputFormatter {
    format: OutputFormat,
    csv_headers_printed: bool,
    log_parser: LogParser,
}

impl OutputFormatter {
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            csv_headers_printed: false,
            log_parser: LogParser::new(),
        }
    }

    pub fn format_line(&mut self, line: &str, colored_line: &str) -> String {
        match self.format {
            OutputFormat::Text => colored_line.to_string(),
            OutputFormat::Json => {
                let parsed = self.log_parser.parse_line(line);
                serde_json::to_string(&parsed).unwrap_or_else(|_| {
                    json!({"raw": line, "error": "failed to parse"}).to_string()
                })
            }
            OutputFormat::Csv => {
                if !self.csv_headers_printed {
                    self.csv_headers_printed = true;
                    let headers = "timestamp,level,service,message,ip,status_code";
                    format!("{}\n{}", headers, self.format_csv_line(line))
                } else {
                    self.format_csv_line(line)
                }
            }
        }
    }

    fn format_csv_line(&self, line: &str) -> String {
        let parsed = self.log_parser.parse_line(line);
        
        let timestamp = parsed["timestamp"].as_str().unwrap_or("");
        let level = parsed["level"].as_str().unwrap_or("");
        let service = parsed["service"].as_str().unwrap_or("");
        let message = parsed["message"].as_str().unwrap_or(line);
        let ip = parsed["ip"].as_str().unwrap_or("");
        let status_code = parsed["status_code"].as_str().unwrap_or("");

        format!(
            "{},{},{},{},{},{}",
            Self::csv_escape(timestamp),
            Self::csv_escape(level),
            Self::csv_escape(service),
            Self::csv_escape(message),
            Self::csv_escape(ip),
            Self::csv_escape(status_code)
        )
    }

    fn csv_escape(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace("\"", "\"\""))
        } else {
            field.to_string()
        }
    }
}

pub struct LogParser {
    timestamp_regex: Regex,
    ip_regex: Regex,
    status_code_regex: Regex,
    level_regex: Regex,
    service_regex: Regex,
}

impl LogParser {
    pub fn new() -> Self {
        Self {
            timestamp_regex: Regex::new(r"\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2}").unwrap(),
            ip_regex: Regex::new(r"\b(?:[0-9]{1,3}\.){3}[0-9]{1,3}\b").unwrap(),
            status_code_regex: Regex::new(r"\b[2-5][0-9]{2}\b").unwrap(),
            level_regex: Regex::new(r"\b(EMERG|ALERT|CRIT|ERROR|WARN|NOTICE|INFO|DEBUG|TRACE)\b").unwrap(),
            service_regex: Regex::new(r"\b(nginx|apache|mysql|postgres|sshd|systemd|docker|php-fpm)\b").unwrap(),
        }
    }

    pub fn parse_line(&self, line: &str) -> Value {
        let mut parsed = HashMap::new();

        // Extract timestamp
        if let Some(ts_match) = self.timestamp_regex.find(line) {
            parsed.insert("timestamp".to_string(), json!(ts_match.as_str()));
        }

        // Extract IP address
        if let Some(ip_match) = self.ip_regex.find(line) {
            parsed.insert("ip".to_string(), json!(ip_match.as_str()));
        }

        // Extract status code
        if let Some(status_match) = self.status_code_regex.find(line) {
            parsed.insert("status_code".to_string(), json!(status_match.as_str()));
        }

        // Extract log level
        if let Some(level_match) = self.level_regex.find(line) {
            parsed.insert("level".to_string(), json!(level_match.as_str()));
        }

        // Extract service
        if let Some(service_match) = self.service_regex.find(line) {
            parsed.insert("service".to_string(), json!(service_match.as_str()));
        }

        // Always include the raw message
        parsed.insert("message".to_string(), json!(line));
        parsed.insert("raw".to_string(), json!(line));

        json!(parsed)
    }
}