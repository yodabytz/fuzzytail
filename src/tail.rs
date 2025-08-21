use crate::config::Config;
use crate::theme::Theme;
use crate::colorizer::Colorizer;
use crate::filter::LineFilter;
use crate::interactive::InteractiveMode;
use crate::output::{OutputFormat, OutputFormatter};
use anyhow::{Context, Result, anyhow};
use std::fs::File;
use std::io::{BufRead, BufReader, stdin, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::collections::VecDeque;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Config as NotifyConfig};
use std::sync::mpsc;
use std::time::Duration;
use std::thread;

pub struct TailProcessor {
    colorizer: Colorizer,
    config: Config,
    filter: LineFilter,
    interactive: bool,
    output_formatter: OutputFormatter,
    buffer_size: usize,
    bytes_mode: Option<usize>,
    quiet: bool,
    verbose: bool,
}

impl TailProcessor {
    pub fn new(
        config: Config, 
        no_color: bool,
        include: Option<String>,
        exclude: Option<String>,
        level: Option<String>,
        interactive: bool,
        format: String,
        buffer_size: usize,
        bytes_mode: Option<usize>,
        quiet: bool,
        verbose: bool,
    ) -> Result<Self> {
        let theme_name = &config.general.theme;
        let theme_path = config.get_theme_path(theme_name)
            .ok_or_else(|| anyhow!("Theme '{}' not found", theme_name))?;
        
        let theme = Theme::load_from_file(&theme_path, theme_name.clone())
            .with_context(|| format!("Failed to load theme from {:?}", theme_path))?;
        
        let colorizer = Colorizer::new(theme, no_color);
        let filter = LineFilter::new(include, exclude, level)?;
        let output_format = OutputFormat::from_string(&format);
        let output_formatter = OutputFormatter::new(output_format);
        
        Ok(Self {
            colorizer,
            config,
            filter,
            interactive,
            output_formatter,
            buffer_size,
            bytes_mode,
            quiet,
            verbose,
        })
    }
    
    pub fn process_stdin(&mut self, lines: usize, follow: bool) -> Result<()> {
        let stdin = stdin();
        let mut reader = BufReader::with_capacity(self.buffer_size, stdin.lock());
        
        if follow {
            // Stream mode - colorize each line as it comes
            for line in reader.lines() {
                let line = line.context("Failed to read from stdin")?;
                if self.filter.should_show_line(&line) {
                    let colored_line = self.colorizer.colorize_line(&line);
                    let formatted = self.output_formatter.format_line(&line, &colored_line);
                    println!("{}", formatted);
                }
            }
        } else {
            // Show last N lines
            let all_lines: Vec<String> = reader.lines()
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to read from stdin")?;
            
            // Apply filter to lines first
            let filtered_lines: Vec<&String> = all_lines.iter()
                .filter(|line| self.filter.should_show_line(line))
                .collect();
            
            let start_idx = filtered_lines.len().saturating_sub(lines);
            for line in &filtered_lines[start_idx..] {
                let colored_line = self.colorizer.colorize_line(line);
                let formatted = self.output_formatter.format_line(line, &colored_line);
                println!("{}", formatted);
            }
        }
        
        Ok(())
    }
    
    pub fn process_files(&mut self, files: &[PathBuf], lines: usize, follow: bool) -> Result<()> {
        if files.len() == 1 {
            self.process_single_file(&files[0], lines, follow)
        } else {
            self.process_multiple_files(files, lines, follow)
        }
    }
    
    fn process_single_file(&mut self, file_path: &Path, lines: usize, follow: bool) -> Result<()> {
        // Show initial lines
        self.show_tail_lines(file_path, lines)?;
        
        if follow {
            self.follow_file(file_path)?;
        }
        
        Ok(())
    }
    
    fn process_multiple_files(&mut self, files: &[PathBuf], lines: usize, follow: bool) -> Result<()> {
        for (i, file_path) in files.iter().enumerate() {
            if i > 0 && !self.quiet {
                println!(); // Blank line between files
            }
            
            if !self.quiet && (self.verbose || files.len() > 1) {
                println!("==> {} <==", file_path.display());
            }
            self.show_tail_lines(file_path, lines)?;
        }
        
        if follow {
            // For multiple files, we need to watch all of them
            self.follow_multiple_files(files)?;
        }
        
        Ok(())
    }
    
    fn show_tail_lines(&mut self, file_path: &Path, lines: usize) -> Result<()> {
        let file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {:?}", file_path))?;
        
        let tail_lines = self.get_last_n_lines(file, lines)?;
        
        // Apply filter and then take last N lines
        let filtered_lines: Vec<&String> = tail_lines.iter()
            .filter(|line| self.filter.should_show_line(line))
            .collect();
        
        let start_idx = filtered_lines.len().saturating_sub(lines);
        for line in &filtered_lines[start_idx..] {
            let colored_line = self.colorizer.colorize_line(line);
            let formatted = self.output_formatter.format_line(line, &colored_line);
            println!("{}", formatted);
        }
        
        Ok(())
    }
    
    fn get_last_n_lines(&self, file: File, n: usize) -> Result<Vec<String>> {
        // Use larger buffer for better performance on large files
        let mut reader = BufReader::with_capacity(self.buffer_size, file);
        let mut all_lines = Vec::new();
        let mut line = String::new();
        
        while reader.read_line(&mut line)? > 0 {
            all_lines.push(line.trim_end_matches('\n').trim_end_matches('\r').to_string());
            line.clear();
        }
        
        let start_idx = all_lines.len().saturating_sub(n);
        Ok(all_lines[start_idx..].to_vec())
    }
    
    fn follow_file(&mut self, file_path: &Path) -> Result<()> {
        let (tx, rx) = mpsc::channel();
        
        let mut watcher: RecommendedWatcher = Watcher::new(tx, NotifyConfig::default())?;
        watcher.watch(file_path, RecursiveMode::NonRecursive)?;
        
        let mut file = File::open(file_path)?;
        let mut pos = file.seek(SeekFrom::End(0))?;
        
        loop {
            // Check for new content
            let current_size = file.seek(SeekFrom::End(0))?;
            if current_size > pos {
                file.seek(SeekFrom::Start(pos))?;
                let mut reader = BufReader::with_capacity(self.buffer_size, &file);
                
                let mut line = String::new();
                while reader.read_line(&mut line)? > 0 {
                    let clean_line = line.trim_end();
                    if self.filter.should_show_line(clean_line) {
                        let colored_line = self.colorizer.colorize_line(clean_line);
                        let formatted = self.output_formatter.format_line(clean_line, &colored_line);
                        println!("{}", formatted);
                    }
                    line.clear();
                }
                
                pos = current_size;
            } else if current_size < pos {
                // File was truncated
                pos = 0;
                file.seek(SeekFrom::Start(0))?;
            }
            
            // Handle file system events
            match rx.try_recv() {
                Ok(_) => {
                    // File changed, continue loop
                }
                Err(mpsc::TryRecvError::Empty) => {
                    // No events, sleep briefly
                    thread::sleep(Duration::from_millis(100));
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err(anyhow!("File watcher disconnected"));
                }
            }
        }
    }
    
    fn follow_multiple_files(&mut self, _files: &[PathBuf]) -> Result<()> {
        // TODO: Implement multi-file following
        // This would require more complex logic to track multiple files
        // and show which file each line comes from
        todo!("Multi-file following not yet implemented")
    }

    pub fn show_default_logs(&mut self, lines: usize) -> Result<()> {
        // Try common system log locations in order of preference
        let default_logs = [
            "/var/log/syslog",
            "/var/log/messages", 
            "/var/log/auth.log",
            "/var/log/kern.log",
            "/var/log/dmesg",
            "test.log", // Our test file for demo
        ];

        println!("🎯 FuzzyTail (ft) - No files specified. Showing available system logs:");
        println!();

        let mut found_logs = Vec::new();
        for log_path in &default_logs {
            let path = PathBuf::from(log_path);
            if path.exists() {
                if let Ok(metadata) = path.metadata() {
                    if metadata.len() > 0 {
                        found_logs.push(path);
                    }
                }
            }
        }

        if found_logs.is_empty() {
            println!("No accessible log files found. Try:");
            println!("  ft /var/log/syslog     # System logs");
            println!("  ft -f myapp.log        # Follow your application log");
            println!("  echo 'test' | ft       # Pipe data to ft");
            println!("  ft --help              # See all options");
            return Ok(());
        }

        // Show the first available log file
        let log_file = &found_logs[0];
        println!("📋 Showing last {} lines from: {}", lines, log_file.display());
        println!("    💡 Use: ft {} -f  to follow this log", log_file.display());
        println!();

        self.show_tail_lines(log_file, lines)?;

        if found_logs.len() > 1 {
            println!();
            println!("📁 Other available logs:");
            for log in &found_logs[1..] {
                println!("    ft {}", log.display());
            }
        }

        println!();
        println!("✨ Try: ft --level ERROR  ft --format json  ft --include \"nginx|mysql\"");

        Ok(())
    }
}