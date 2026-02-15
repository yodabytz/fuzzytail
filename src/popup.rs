use anyhow::Result;
use crossterm::{
    cursor::MoveTo,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{size, BeginSynchronizedUpdate, EndSynchronizedUpdate},
    event::{poll, read, Event, KeyCode, KeyModifiers},
    queue,
};
use std::io::{self, Write};
use std::time::Duration;

use crate::tail::theme_color_to_ansi256;

pub struct PopupColors {
    pub border_fg: Color,
    pub border_bg: Color,
    pub content_fg: Color,
    pub content_bg: Color,
    pub highlight_fg: Color,
    pub highlight_bg: Color,
}

pub enum PopupResult {
    Selected(usize),
    Text(String),
    Dismissed,
}

impl PopupColors {
    pub fn from_theme(theme: &crate::theme::Theme) -> Self {
        let bar_bg = theme_color_to_ansi256(theme.statusbar_bg.as_ref(), 103);
        let bar_fg = theme_color_to_ansi256(theme.statusbar_fg.as_ref(), 231);

        PopupColors {
            border_fg: bar_fg,
            border_bg: bar_bg,
            content_fg: Color::AnsiValue(231), // white
            content_bg: Color::AnsiValue(236), // dark grey
            highlight_fg: Color::AnsiValue(232), // black
            highlight_bg: Color::AnsiValue(117), // light blue
        }
    }
}

/// Calculate centered position for a popup of given dimensions.
fn center_popup(term_w: u16, term_h: u16, popup_w: u16, popup_h: u16) -> (u16, u16) {
    let x = term_w.saturating_sub(popup_w) / 2;
    let y = term_h.saturating_sub(popup_h) / 2;
    (x, y)
}

/// Draw a bordered popup frame with a title into the buffer.
fn draw_popup_frame(
    buf: &mut Vec<u8>,
    x: u16,
    y: u16,
    w: u16,
    h: u16,
    title: &str,
    colors: &PopupColors,
) -> Result<()> {
    let inner_w = w.saturating_sub(2) as usize;

    // Top border: ┌─ Title ─┐
    queue!(buf, MoveTo(x, y), SetForegroundColor(colors.border_fg), SetBackgroundColor(colors.border_bg))?;
    let title_display = if title.len() > inner_w.saturating_sub(2) {
        &title[..inner_w.saturating_sub(2)]
    } else {
        title
    };
    let left_pad = (inner_w.saturating_sub(title_display.len())) / 2;
    let right_pad = inner_w.saturating_sub(title_display.len()).saturating_sub(left_pad);
    let top = format!("┌{}{}{}┐",
        "─".repeat(left_pad),
        title_display,
        "─".repeat(right_pad),
    );
    queue!(buf, Print(&top))?;

    // Side borders + content background
    for row in 1..h.saturating_sub(1) {
        queue!(buf, MoveTo(x, y + row), SetForegroundColor(colors.border_fg), SetBackgroundColor(colors.border_bg), Print("│"))?;
        queue!(buf, SetForegroundColor(colors.content_fg), SetBackgroundColor(colors.content_bg))?;
        queue!(buf, Print(" ".repeat(inner_w)))?;
        queue!(buf, SetForegroundColor(colors.border_fg), SetBackgroundColor(colors.border_bg), Print("│"))?;
    }

    // Bottom border: └──┘
    queue!(buf, MoveTo(x, y + h - 1), SetForegroundColor(colors.border_fg), SetBackgroundColor(colors.border_bg))?;
    let bottom = format!("└{}┘", "─".repeat(inner_w));
    queue!(buf, Print(&bottom), ResetColor)?;

    Ok(())
}

/// Display an info popup. Any key dismisses it.
pub fn popup_info(title: &str, lines: &[String], colors: &PopupColors) -> Result<()> {
    let (tw, th) = size()?;
    let max_line_len = lines.iter().map(|l| l.len()).max().unwrap_or(10);
    let popup_w = (max_line_len + 4).min(tw as usize - 4) as u16;
    let popup_h = (lines.len() as u16 + 2).min(th - 2);
    let (px, py) = center_popup(tw, th, popup_w, popup_h);
    let inner_w = (popup_w - 2) as usize;

    let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
    queue!(buf, BeginSynchronizedUpdate)?;
    draw_popup_frame(&mut buf, px, py, popup_w, popup_h, title, colors)?;

    let visible_lines = (popup_h - 2) as usize;
    for (i, line) in lines.iter().take(visible_lines).enumerate() {
        queue!(buf, MoveTo(px + 1, py + 1 + i as u16),
            SetForegroundColor(colors.content_fg), SetBackgroundColor(colors.content_bg))?;
        let display = if line.len() > inner_w {
            &line[..inner_w]
        } else {
            line
        };
        let padded = format!("{:<width$}", display, width = inner_w);
        queue!(buf, Print(padded))?;
    }

    queue!(buf, ResetColor, EndSynchronizedUpdate)?;
    let mut stdout = io::stdout().lock();
    stdout.write_all(&buf)?;
    stdout.flush()?;

    // Wait for any key
    loop {
        if poll(Duration::from_millis(100))? {
            if let Event::Key(_) = read()? {
                break;
            }
        }
    }

    Ok(())
}

/// Display a menu popup with arrow key navigation. Returns Selected(index) or Dismissed.
pub fn popup_menu(title: &str, items: &[String], colors: &PopupColors) -> Result<PopupResult> {
    if items.is_empty() {
        return Ok(PopupResult::Dismissed);
    }

    let (tw, th) = size()?;
    let max_item_len = items.iter().map(|l| l.len()).max().unwrap_or(10);
    let popup_w = (max_item_len + 6).min(tw as usize - 4) as u16; // +6 for borders + "> " prefix
    let popup_h = (items.len() as u16 + 2).min(th - 2);
    let (px, py) = center_popup(tw, th, popup_w, popup_h);
    let inner_w = (popup_w - 2) as usize;

    let visible_count = (popup_h - 2) as usize;
    let mut selected: usize = 0;
    let mut scroll_offset: usize = 0;

    loop {
        let mut buf: Vec<u8> = Vec::with_capacity(8 * 1024);
        queue!(buf, BeginSynchronizedUpdate)?;
        draw_popup_frame(&mut buf, px, py, popup_w, popup_h, title, colors)?;

        // Adjust scroll so selected is visible
        if selected < scroll_offset {
            scroll_offset = selected;
        } else if selected >= scroll_offset + visible_count {
            scroll_offset = selected - visible_count + 1;
        }

        for i in 0..visible_count {
            let item_idx = scroll_offset + i;
            queue!(buf, MoveTo(px + 1, py + 1 + i as u16))?;

            if item_idx < items.len() {
                if item_idx == selected {
                    queue!(buf, SetForegroundColor(colors.highlight_fg), SetBackgroundColor(colors.highlight_bg))?;
                    let display = format!("> {}", &items[item_idx]);
                    let truncated = if display.len() > inner_w { &display[..inner_w] } else { &display };
                    let padded = format!("{:<width$}", truncated, width = inner_w);
                    queue!(buf, Print(padded))?;
                } else {
                    queue!(buf, SetForegroundColor(colors.content_fg), SetBackgroundColor(colors.content_bg))?;
                    let display = format!("  {}", &items[item_idx]);
                    let truncated = if display.len() > inner_w { &display[..inner_w] } else { &display };
                    let padded = format!("{:<width$}", truncated, width = inner_w);
                    queue!(buf, Print(padded))?;
                }
            } else {
                queue!(buf, SetForegroundColor(colors.content_fg), SetBackgroundColor(colors.content_bg))?;
                queue!(buf, Print(" ".repeat(inner_w)))?;
            }
        }

        queue!(buf, ResetColor, EndSynchronizedUpdate)?;
        let mut stdout = io::stdout().lock();
        stdout.write_all(&buf)?;
        stdout.flush()?;

        // Read input
        if poll(Duration::from_millis(100))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Up => {
                        if selected > 0 { selected -= 1; }
                    }
                    KeyCode::Down => {
                        if selected < items.len() - 1 { selected += 1; }
                    }
                    KeyCode::Home => { selected = 0; }
                    KeyCode::End => { selected = items.len() - 1; }
                    KeyCode::Enter => {
                        return Ok(PopupResult::Selected(selected));
                    }
                    KeyCode::Esc => {
                        return Ok(PopupResult::Dismissed);
                    }
                    KeyCode::Char('q') => {
                        return Ok(PopupResult::Dismissed);
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(PopupResult::Dismissed);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Display a text input popup. Returns Text(string) or Dismissed.
pub fn popup_input(title: &str, prompt: &str, default: &str, colors: &PopupColors) -> Result<PopupResult> {
    let (tw, th) = size()?;
    let popup_w = 50u16.min(tw - 4);
    let popup_h = 4u16; // border top + prompt + input + border bottom
    let (px, py) = center_popup(tw, th, popup_w, popup_h);
    let inner_w = (popup_w - 2) as usize;

    let mut input = default.to_string();
    let mut cursor_pos = input.len();

    loop {
        let mut buf: Vec<u8> = Vec::with_capacity(4 * 1024);
        queue!(buf, BeginSynchronizedUpdate)?;
        draw_popup_frame(&mut buf, px, py, popup_w, popup_h, title, colors)?;

        // Prompt line
        queue!(buf, MoveTo(px + 1, py + 1),
            SetForegroundColor(colors.content_fg), SetBackgroundColor(colors.content_bg))?;
        let prompt_display = if prompt.len() > inner_w { &prompt[..inner_w] } else { prompt };
        let padded_prompt = format!("{:<width$}", prompt_display, width = inner_w);
        queue!(buf, Print(padded_prompt))?;

        // Input line with cursor
        queue!(buf, MoveTo(px + 1, py + 2),
            SetForegroundColor(colors.highlight_fg), SetBackgroundColor(colors.highlight_bg))?;
        let visible_input = if input.len() > inner_w { &input[input.len() - inner_w..] } else { &input };
        let padded_input = format!("{:<width$}", visible_input, width = inner_w);
        queue!(buf, Print(padded_input))?;

        // Show cursor position
        let cursor_x = if cursor_pos > inner_w { inner_w } else { cursor_pos };
        queue!(buf, MoveTo(px + 1 + cursor_x as u16, py + 2))?;

        queue!(buf, ResetColor, EndSynchronizedUpdate)?;
        let mut stdout = io::stdout().lock();
        stdout.write_all(&buf)?;
        stdout.flush()?;

        if poll(Duration::from_millis(100))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Enter => {
                        return Ok(PopupResult::Text(input));
                    }
                    KeyCode::Esc => {
                        return Ok(PopupResult::Dismissed);
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(PopupResult::Dismissed);
                    }
                    KeyCode::Backspace => {
                        if cursor_pos > 0 {
                            input.remove(cursor_pos - 1);
                            cursor_pos -= 1;
                        }
                    }
                    KeyCode::Delete => {
                        if cursor_pos < input.len() {
                            input.remove(cursor_pos);
                        }
                    }
                    KeyCode::Left => {
                        if cursor_pos > 0 { cursor_pos -= 1; }
                    }
                    KeyCode::Right => {
                        if cursor_pos < input.len() { cursor_pos += 1; }
                    }
                    KeyCode::Home => { cursor_pos = 0; }
                    KeyCode::End => { cursor_pos = input.len(); }
                    KeyCode::Char(c) => {
                        input.insert(cursor_pos, c);
                        cursor_pos += 1;
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Wrapper around popup_menu for window/file selection.
pub fn popup_select_window(names: &[String], colors: &PopupColors) -> Result<PopupResult> {
    popup_menu(" Select Window ", names, colors)
}
