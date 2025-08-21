use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use anyhow::Result;
use std::io;
use crate::colorizer::Colorizer;
use crate::filter::LineFilter;

pub struct InteractiveMode {
    lines: Vec<String>,
    current_line: usize,
    paused: bool,
    colorizer: Colorizer,
    filter: LineFilter,
}

impl InteractiveMode {
    pub fn new(lines: Vec<String>, colorizer: Colorizer, filter: LineFilter) -> Self {
        Self {
            lines,
            current_line: 0,
            paused: false,
            colorizer,
            filter,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let result = self.run_app();

        // Restore terminal
        disable_raw_mode()?;
        execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

        result
    }

    fn run_app(&mut self) -> Result<()> {
        loop {
            self.draw()?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if self.handle_key_event(key)? {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(true),
            KeyCode::Char(' ') => self.paused = !self.paused,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.current_line > 0 {
                    self.current_line -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.current_line < self.lines.len().saturating_sub(1) {
                    self.current_line += 1;
                }
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.current_line = 0;
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.current_line = self.lines.len().saturating_sub(1);
            }
            KeyCode::PageUp => {
                self.current_line = self.current_line.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.current_line = (self.current_line + 10).min(self.lines.len().saturating_sub(1));
            }
            _ => {}
        }
        Ok(false)
    }

    fn draw(&self) -> Result<()> {
        use crossterm::{
            cursor::MoveTo,
            style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
            terminal::{Clear, ClearType, size},
        };

        let (width, height) = size()?;
        let content_height = height as usize - 2; // Reserve space for status line

        // Clear screen
        execute!(io::stdout(), Clear(ClearType::All))?;

        // Show visible lines
        let start_line = self.current_line.saturating_sub(content_height / 2);
        let end_line = (start_line + content_height).min(self.lines.len());

        for (i, line_idx) in (start_line..end_line).enumerate() {
            if let Some(line) = self.lines.get(line_idx) {
                execute!(io::stdout(), MoveTo(0, i as u16))?;
                
                // Highlight current line
                if line_idx == self.current_line {
                    execute!(io::stdout(), SetBackgroundColor(Color::DarkGrey))?;
                }

                // Apply filter and colorization
                if self.filter.should_show_line(line) {
                    let colored_line = self.colorizer.colorize_line(line);
                    execute!(io::stdout(), Print(&colored_line))?;
                } else {
                    execute!(io::stdout(), SetForegroundColor(Color::DarkGrey))?;
                    execute!(io::stdout(), Print(&format!("(filtered) {}", line)))?;
                }

                execute!(io::stdout(), ResetColor)?;
            }
        }

        // Status line
        let status = format!(
            " Line {}/{} | {} | Press 'q' to quit, SPACE to pause, arrows to navigate ",
            self.current_line + 1,
            self.lines.len(),
            if self.paused { "PAUSED" } else { "RUNNING" }
        );
        
        execute!(io::stdout(), MoveTo(0, height - 1))?;
        execute!(io::stdout(), SetBackgroundColor(Color::Blue))?;
        execute!(io::stdout(), SetForegroundColor(Color::White))?;
        execute!(io::stdout(), Print(&status))?;
        
        // Fill remaining space on status line
        let remaining = width as usize - status.len();
        if remaining > 0 {
            execute!(io::stdout(), Print(&" ".repeat(remaining)))?;
        }
        
        execute!(io::stdout(), ResetColor)?;

        Ok(())
    }
}