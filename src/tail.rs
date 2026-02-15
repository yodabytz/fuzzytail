use crate::config::Config;
use crate::theme::Theme;
use crate::colorizer::Colorizer;
use crate::filter::LineFilter;
use crate::output::{OutputFormat, OutputFormatter};
use anyhow::{Context, Result, anyhow};
use std::fs::File;
use std::io::{BufRead, BufReader, stdin, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::collections::VecDeque;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Config as NotifyConfig};
use std::sync::mpsc;
use std::time::Duration;
use std::thread;
use crossterm::{
    cursor::{Hide, Show, MoveTo},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType, size, EnterAlternateScreen, LeaveAlternateScreen,
               BeginSynchronizedUpdate, EndSynchronizedUpdate},
    event::{poll, read, Event, KeyCode, KeyModifiers},
    execute, queue,
};
use std::io::{self, Write};

struct FileTracker {
    path: PathBuf,
    file: File,
    position: u64,
    lines: VecDeque<String>,
    max_lines: usize,
    line_count: usize,
    last_update: std::time::SystemTime,
    file_id: Option<(u64, u64)>,
}

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
        let reader = BufReader::with_capacity(self.buffer_size, stdin.lock());

        if follow {
            for line in reader.lines() {
                let line = line.context("Failed to read from stdin")?;
                if self.filter.should_show_line(&line) {
                    let colored_line = self.colorizer.colorize_line(&line);
                    let formatted = self.output_formatter.format_line(&line, &colored_line);
                    println!("{}", formatted);
                }
            }
        } else {
            let all_lines: Vec<String> = reader.lines()
                .collect::<Result<Vec<_>, _>>()
                .context("Failed to read from stdin")?;

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
        self.show_tail_lines(file_path, lines)?;

        if follow {
            self.follow_file(file_path)?;
        }

        Ok(())
    }

    fn process_multiple_files(&mut self, files: &[PathBuf], lines: usize, follow: bool) -> Result<()> {
        if follow {
            self.follow_multiple_files(files)?;
        } else {
            for (i, file_path) in files.iter().enumerate() {
                if i > 0 && !self.quiet {
                    println!();
                }

                if !self.quiet && (self.verbose || files.len() > 1) {
                    println!("==> {} <==", file_path.display());
                }
                self.show_tail_lines(file_path, lines)?;
            }
        }

        Ok(())
    }

    fn show_tail_lines(&mut self, file_path: &Path, lines: usize) -> Result<()> {
        let file = File::open(file_path)
            .with_context(|| format!("Failed to open file: {:?}", file_path))?;

        let tail_lines = self.get_last_n_lines(file, lines)?;

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
        let mut file_id = get_open_file_id(&file);

        loop {
            // Check for log rotation (file at path replaced with new inode)
            if let (Some(ref current_id), Some(path_id)) = (&file_id, get_file_id(file_path)) {
                if *current_id != path_id {
                    // Drain remaining data from old (rotated) file
                    let old_size = file.metadata().map(|m| m.len()).unwrap_or(pos);
                    if old_size > pos {
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
                    }

                    // Reopen the new file at the same path
                    match File::open(file_path) {
                        Ok(new_file) => {
                            file = new_file;
                            pos = 0;
                            file_id = get_open_file_id(&file);
                            let _ = watcher.unwatch(file_path);
                            let _ = watcher.watch(file_path, RecursiveMode::NonRecursive);
                        }
                        Err(_) => {
                            // New file not yet created, try next cycle
                            thread::sleep(Duration::from_millis(100));
                            continue;
                        }
                    }
                }
            }

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
                // File truncated in place (e.g., copytruncate)
                pos = 0;
                file.seek(SeekFrom::Start(0))?;
            }

            match rx.try_recv() {
                Ok(_) => {}
                Err(_) => {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }
    }

    fn follow_multiple_files(&mut self, files: &[PathBuf]) -> Result<()> {
        use crossterm::terminal::{enable_raw_mode, disable_raw_mode};

        if let Err(_) = enable_raw_mode() {
            return self.follow_multiple_files_scroll(files);
        }

        let mut stdout = io::stdout();
        if let Err(_) = execute!(stdout, EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return self.follow_multiple_files_scroll(files);
        }

        let result = self.follow_multiple_files_panes(files);

        let _ = execute!(stdout, Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
        result
    }

    fn follow_multiple_files_panes(&mut self, files: &[PathBuf]) -> Result<()> {
        let mut file_trackers: Vec<FileTracker> = Vec::new();

        for file_path in files {
            let file = File::open(file_path)
                .with_context(|| format!("Failed to open file: {:?}", file_path))?;
            let pos = file.metadata()?.len();
            let file_id = get_open_file_id(&file);

            let mut tracker = FileTracker {
                path: file_path.clone(),
                file,
                position: pos,
                lines: VecDeque::new(),
                max_lines: 200,
                line_count: 0,
                last_update: std::time::SystemTime::now(),
                file_id,
            };

            tracker.line_count = self.count_lines_in_file(&tracker.path).unwrap_or(0);

            if let Ok(initial_lines) = self.get_last_n_lines(File::open(file_path)?, 100) {
                for line in initial_lines {
                    if self.filter.should_show_line(&line) {
                        let colored_line = self.colorizer.colorize_line(&line);
                        tracker.lines.push_back(colored_line);
                    }
                }
            }

            file_trackers.push(tracker);
        }

        let (tx, rx) = mpsc::channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, NotifyConfig::default())?;

        for file_path in files {
            watcher.watch(file_path, RecursiveMode::NonRecursive)?;
        }

        // Initial render
        self.render_frame(&file_trackers)?;

        loop {
            // Drain all pending watcher events (non-blocking)
            loop {
                match rx.try_recv() {
                    Ok(_) => {}
                    Err(_) => break,
                }
            }

            // Check for new content and log rotation
            let mut content_changed = false;
            for tracker in &mut file_trackers {
                let old_len = tracker.lines.len();
                let rotated = self.check_file_updates(tracker)?;
                if tracker.lines.len() != old_len {
                    content_changed = true;
                }
                if rotated {
                    content_changed = true;
                    // Re-watch the new file at the same path
                    let _ = watcher.unwatch(&tracker.path);
                    let _ = watcher.watch(&tracker.path, RecursiveMode::NonRecursive);
                }
            }

            if content_changed {
                self.render_frame(&file_trackers)?;
            }

            // Check keyboard
            if poll(Duration::from_millis(0))? {
                if let Event::Key(key) = read()? {
                    match key.code {
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Char('h') | KeyCode::F(1) => {
                            self.show_help(&file_trackers)?;
                            self.render_frame(&file_trackers)?;
                        }
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            let file_num = c.to_digit(10).unwrap() as usize;
                            if file_num > 0 && file_num <= file_trackers.len() {
                                self.show_single_pane(&file_trackers[file_num - 1], file_num - 1)?;
                                self.render_frame(&file_trackers)?;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Sleep to avoid busy-waiting
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    /// Render the entire screen as one atomic frame.
    /// All output goes into a Vec<u8>, then one write_all + flush.
    /// Synchronized update markers tell the terminal to display atomically.
    fn render_frame(&self, trackers: &[FileTracker]) -> Result<()> {
        let (tw, th) = size()?;
        let num_files = trackers.len() as u16;
        if th < num_files * 2 || tw < 10 {
            return Ok(());
        }

        // All rows divided among panes (each pane = content + status bar at bottom)
        let base_h = th / num_files;
        let extra = th % num_files;

        let mut buf: Vec<u8> = Vec::with_capacity(64 * 1024);
        queue!(buf, BeginSynchronizedUpdate)?;

        let mut y = 0u16;
        for (i, tracker) in trackers.iter().enumerate() {
            let h = if i as u16 == num_files - 1 { base_h + extra } else { base_h };
            self.write_pane(&mut buf, tracker, i, y, tw, h)?;
            y += h;
        }

        queue!(buf, EndSynchronizedUpdate)?;

        let mut stdout = io::stdout().lock();
        stdout.write_all(&buf)?;
        stdout.flush()?;
        Ok(())
    }

    /// Write a single pane: content lines first, status bar at bottom (like multitail).
    /// Every line is padded to full width — no Clear escape codes needed.
    fn write_pane(&self, buf: &mut Vec<u8>, tracker: &FileTracker, index: usize, y: u16, width: u16, height: u16) -> Result<()> {
        let w = width as usize;

        // Content area is everything except the last row (status bar)
        let content_h = height.saturating_sub(1) as usize;
        let start = tracker.lines.len().saturating_sub(content_h);
        let visible: Vec<&String> = tracker.lines.iter().skip(start).take(content_h).collect();

        // Draw content lines, each padded to full width (overwrites old content, no flicker)
        for i in 0..content_h {
            let row = y + i as u16;
            queue!(buf, MoveTo(0, row))?;
            if i < visible.len() {
                let padded = pad_ansi(visible[i], w);
                queue!(buf, Print(padded))?;
            } else {
                queue!(buf, Print(" ".repeat(w)))?;
            }
        }

        // Status bar colors from theme (xterm-256 only for compatibility)
        let theme = self.colorizer.get_theme();
        let bg = theme_color_to_ansi256(theme.statusbar_bg.as_ref(), 103);
        let fg = theme_color_to_ansi256(theme.statusbar_fg.as_ref(), 231);

        // Status bar at the bottom of this pane (like multitail)
        let filepath = tracker.path.to_string_lossy();
        let now = std::time::SystemTime::now();
        let datetime = chrono::DateTime::<chrono::Local>::from(now);
        let time_str = datetime.format("%b %d %H:%M:%S %Y").to_string();

        let left = format!("{}] {}", index, filepath);
        let right = format!("{} - {}", tracker.line_count, time_str);
        let gap = w.saturating_sub(left.len() + right.len());
        let mut bar = left;
        if gap > 0 {
            bar.push_str(&" ".repeat(gap));
        }
        bar.push_str(&right);
        if bar.len() > w { bar.truncate(w); }
        while bar.len() < w { bar.push(' '); }

        let bar_row = y + height - 1;
        queue!(
            buf,
            MoveTo(0, bar_row),
            SetBackgroundColor(bg),
            SetForegroundColor(fg),
            Print(&bar),
            ResetColor
        )?;

        Ok(())
    }

    fn show_help(&self, _file_trackers: &[FileTracker]) -> Result<()> {
        let mut stdout = io::stdout();
        let (width, _) = crossterm::terminal::size().unwrap_or((80, 24));

        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0), Show)?;

        execute!(stdout, SetBackgroundColor(Color::Blue), SetForegroundColor(Color::White))?;
        let title = format!("{:^w$}", "FuzzyTail Help", w = width as usize);
        execute!(stdout, Print(&title), ResetColor)?;
        println!();

        println!();
        println!("KEYBOARD COMMANDS:");
        println!("  h / F1       - This help screen");
        println!("  q / ESC      - Quit the program");
        println!("  Ctrl+C       - Emergency exit");
        println!("  1-9          - View single file full-screen");
        println!();
        println!("COMMAND LINE OPTIONS:");
        println!("  ft <files>           - Tail files (auto-follow with multiple files)");
        println!("  ft -f <file>         - Follow a single file");
        println!("  ft -n 50 <file>      - Show last 50 lines");
        println!("  ft --exclude <pat>   - Exclude lines matching pattern");
        println!("  ft --include <pat>   - Show only lines matching pattern");
        println!("  ft --level ERROR     - Filter by log level");
        println!("  ft --no-color        - Disable color output");
        println!("  ft --format json     - Output as JSON");
        println!();
        println!("FEATURES:");
        println!("  Real-time log file monitoring");
        println!("  Split-pane display for multiple files");
        println!("  Theme-based syntax highlighting");
        println!("  Regular expression filtering");
        println!("  Log level filtering");
        println!();

        execute!(stdout, SetBackgroundColor(Color::DarkGrey), SetForegroundColor(Color::White))?;
        let footer = format!("{:^w$}", "Press any key to return...", w = width as usize);
        execute!(stdout, Print(&footer), ResetColor)?;
        stdout.flush()?;

        loop {
            if poll(Duration::from_millis(100))? {
                if let Event::Key(_) = read()? {
                    break;
                }
            }
        }

        execute!(stdout, Clear(ClearType::All), Hide, MoveTo(0, 0))?;
        Ok(())
    }

    fn show_single_pane(&self, tracker: &FileTracker, index: usize) -> Result<()> {
        let (width, height) = size()?;

        let mut buf: Vec<u8> = Vec::with_capacity(16 * 1024);
        queue!(buf, BeginSynchronizedUpdate)?;
        self.write_pane(&mut buf, tracker, index, 0, width, height)?;
        queue!(buf, EndSynchronizedUpdate)?;

        let mut stdout = io::stdout().lock();
        stdout.write_all(&buf)?;
        stdout.flush()?;

        loop {
            if poll(Duration::from_millis(100))? {
                if let Event::Key(_) = read()? {
                    break;
                }
            }
        }

        Ok(())
    }

    fn follow_multiple_files_scroll(&mut self, files: &[PathBuf]) -> Result<()> {
        let mut file_trackers: Vec<FileTracker> = Vec::new();

        for file_path in files {
            let file = File::open(file_path)
                .with_context(|| format!("Failed to open file: {:?}", file_path))?;
            let pos = file.metadata()?.len();
            let file_id = get_open_file_id(&file);

            let mut tracker = FileTracker {
                path: file_path.clone(),
                file,
                position: pos,
                lines: VecDeque::new(),
                max_lines: 10,
                line_count: 0,
                last_update: std::time::SystemTime::now(),
                file_id,
            };

            if let Ok(initial_lines) = self.get_last_n_lines(File::open(file_path)?, 5) {
                for line in initial_lines {
                    if self.filter.should_show_line(&line) {
                        let colored_line = self.colorizer.colorize_line(&line);
                        tracker.lines.push_back(colored_line);
                    }
                }
            }

            file_trackers.push(tracker);
        }

        let (tx, rx) = mpsc::channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, NotifyConfig::default())?;

        for file_path in files {
            watcher.watch(file_path, RecursiveMode::NonRecursive)?;
        }

        for tracker in &file_trackers {
            let filename = tracker.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            println!("==> {} <==", filename);
            for line in &tracker.lines {
                println!("{}", line);
            }
            println!();
        }

        loop {
            for tracker in &mut file_trackers {
                let filename = tracker.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Check for log rotation
                let mut was_rotated = false;
                if let (Some(ref open_id), Some(path_id)) = (&tracker.file_id, get_file_id(&tracker.path)) {
                    if *open_id != path_id {
                        // Drain remaining data from old (rotated) file
                        let old_size = tracker.file.metadata().map(|m| m.len()).unwrap_or(tracker.position);
                        if old_size > tracker.position {
                            tracker.file.seek(SeekFrom::Start(tracker.position))?;
                            let mut reader = BufReader::with_capacity(self.buffer_size, &tracker.file);
                            let mut line = String::new();
                            while reader.read_line(&mut line)? > 0 {
                                let clean_line = line.trim_end().to_string();
                                if self.filter.should_show_line(&clean_line) {
                                    let colored_line = self.colorizer.colorize_line(&clean_line);
                                    println!("[{}] {}", filename, colored_line);
                                }
                                line.clear();
                            }
                            tracker.position = old_size;
                        }

                        // Reopen the new file
                        if let Ok(new_file) = File::open(&tracker.path) {
                            tracker.file = new_file;
                            tracker.position = 0;
                            tracker.file_id = get_open_file_id(&tracker.file);
                            let _ = watcher.unwatch(&tracker.path);
                            let _ = watcher.watch(&tracker.path, RecursiveMode::NonRecursive);
                            was_rotated = true;
                        }
                    }
                }

                let current_size = tracker.file.metadata()?.len();

                if current_size > tracker.position {
                    tracker.file.seek(SeekFrom::Start(tracker.position))?;
                    let mut reader = BufReader::with_capacity(self.buffer_size, &tracker.file);

                    let mut line = String::new();
                    while reader.read_line(&mut line)? > 0 {
                        let clean_line = line.trim_end().to_string();
                        if self.filter.should_show_line(&clean_line) {
                            let colored_line = self.colorizer.colorize_line(&clean_line);
                            println!("[{}] {}", filename, colored_line);
                        }
                        line.clear();
                    }

                    tracker.position = current_size;
                } else if current_size < tracker.position && !was_rotated {
                    // File truncated in place
                    tracker.position = 0;
                    tracker.file.seek(SeekFrom::Start(0))?;
                }
            }

            match rx.try_recv() {
                Ok(_) => {}
                Err(_) => {
                    thread::sleep(Duration::from_millis(100));
                }
            }
        }

        Ok(())
    }

    fn count_lines_in_file(&self, path: &PathBuf) -> Result<usize> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().count())
    }

    /// Check for new content and log rotation. Returns true if the file was rotated.
    fn check_file_updates(&mut self, tracker: &mut FileTracker) -> Result<bool> {
        let mut rotated = false;

        // Check for log rotation: file at path has different inode than our open handle
        if let (Some(ref open_id), Some(path_id)) = (&tracker.file_id, get_file_id(&tracker.path)) {
            if *open_id != path_id {
                // Drain remaining data from old (rotated) file before switching
                let old_size = tracker.file.metadata().map(|m| m.len()).unwrap_or(tracker.position);
                if old_size > tracker.position {
                    tracker.file.seek(SeekFrom::Start(tracker.position))?;
                    let mut reader = BufReader::with_capacity(self.buffer_size, &tracker.file);
                    let mut line = String::new();
                    while reader.read_line(&mut line)? > 0 {
                        let clean_line = line.trim_end().to_string();
                        if self.filter.should_show_line(&clean_line) {
                            let colored_line = self.colorizer.colorize_line(&clean_line);
                            tracker.lines.push_back(colored_line);
                            tracker.line_count += 1;
                            tracker.last_update = std::time::SystemTime::now();
                            while tracker.lines.len() > tracker.max_lines {
                                tracker.lines.pop_front();
                            }
                        }
                        line.clear();
                    }
                    tracker.position = old_size;
                }

                // Reopen the new file at the same path
                match File::open(&tracker.path) {
                    Ok(new_file) => {
                        tracker.file = new_file;
                        tracker.position = 0;
                        tracker.file_id = get_open_file_id(&tracker.file);
                        rotated = true;
                        // Fall through to read new content below
                    }
                    Err(_) => {
                        // New file not yet created (brief window during rotation)
                        return Ok(false);
                    }
                }
            }
        }

        let current_size = tracker.file.metadata()?.len();

        if current_size > tracker.position {
            tracker.file.seek(SeekFrom::Start(tracker.position))?;
            let mut reader = BufReader::with_capacity(self.buffer_size, &tracker.file);

            let mut line = String::new();
            while reader.read_line(&mut line)? > 0 {
                let clean_line = line.trim_end().to_string();
                if self.filter.should_show_line(&clean_line) {
                    let colored_line = self.colorizer.colorize_line(&clean_line);
                    tracker.lines.push_back(colored_line);
                    tracker.line_count += 1;
                    tracker.last_update = std::time::SystemTime::now();

                    while tracker.lines.len() > tracker.max_lines {
                        tracker.lines.pop_front();
                    }
                }
                line.clear();
            }

            tracker.position = current_size;
        } else if current_size < tracker.position && !rotated {
            // File truncated in place (e.g., logrotate copytruncate)
            tracker.position = 0;
            tracker.lines.clear();
            tracker.line_count = 0;
            tracker.file.seek(SeekFrom::Start(0))?;
        }

        Ok(rotated)
    }

    pub fn show_default_logs(&mut self, lines: usize) -> Result<()> {
        let default_logs = [
            "/var/log/syslog",
            "/var/log/messages",
            "/var/log/auth.log",
            "/var/log/kern.log",
            "/var/log/dmesg",
        ];

        println!("ft - No files specified. Showing available system logs:");
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

        let log_file = &found_logs[0];
        println!("Showing last {} lines from: {}", lines, log_file.display());
        println!("  Tip: ft {} -f  to follow this log", log_file.display());
        println!();

        self.show_tail_lines(log_file, lines)?;

        if found_logs.len() > 1 {
            println!();
            println!("Other available logs:");
            for log in &found_logs[1..] {
                println!("  ft {}", log.display());
            }
        }

        Ok(())
    }
}

/// Get the filesystem identity (device, inode) of a file path for rotation detection.
#[cfg(unix)]
fn get_file_id(path: &Path) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    std::fs::metadata(path).ok().map(|m| (m.dev(), m.ino()))
}

#[cfg(not(unix))]
fn get_file_id(_path: &Path) -> Option<(u64, u64)> {
    None
}

/// Get the filesystem identity of an already-open file handle.
#[cfg(unix)]
fn get_open_file_id(file: &File) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    file.metadata().ok().map(|m| (m.dev(), m.ino()))
}

#[cfg(not(unix))]
fn get_open_file_id(_file: &File) -> Option<(u64, u64)> {
    None
}

/// Convert theme Color to crossterm AnsiValue (256-color).
/// Always uses AnsiValue for maximum terminal compatibility.
fn theme_color_to_ansi256(c: Option<&crate::theme::Color>, default: u8) -> Color {
    match c {
        Some(crate::theme::Color::Xterm256(n)) => Color::AnsiValue(*n),
        Some(crate::theme::Color::TrueColor { r, g, b }) => {
            // Approximate RGB to nearest xterm-256 color
            Color::AnsiValue(rgb_to_xterm256(*r, *g, *b))
        }
        None => Color::AnsiValue(default),
    }
}

/// Approximate an RGB color to the nearest xterm-256 color (16-231 cube).
fn rgb_to_xterm256(r: u8, g: u8, b: u8) -> u8 {
    let ri = ((r as u16 * 5 + 127) / 255) as u8;
    let gi = ((g as u16 * 5 + 127) / 255) as u8;
    let bi = ((b as u16 * 5 + 127) / 255) as u8;
    16 + 36 * ri + 6 * gi + bi
}

/// Truncate an ANSI-colored string to `max_width` visible chars, then pad with spaces
/// to exactly `max_width`. No Clear escape codes needed — full overwrite.
fn pad_ansi(s: &str, max_width: usize) -> String {
    let mut result = String::with_capacity(s.len() + max_width);
    let mut visible = 0usize;
    let mut in_escape = false;

    for ch in s.chars() {
        if in_escape {
            result.push(ch);
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }

        if ch == '\x1b' {
            if visible >= max_width {
                break;
            }
            in_escape = true;
            result.push(ch);
            continue;
        }

        if ch == '\t' {
            let spaces = 4 - (visible % 4);
            for _ in 0..spaces {
                if visible >= max_width { break; }
                result.push(' ');
                visible += 1;
            }
            continue;
        }

        if visible >= max_width {
            break;
        }

        if !ch.is_control() {
            result.push(ch);
            visible += 1;
        }
    }

    // Reset color, then pad remaining width with spaces
    result.push_str("\x1b[0m");
    while visible < max_width {
        result.push(' ');
        visible += 1;
    }

    result
}
