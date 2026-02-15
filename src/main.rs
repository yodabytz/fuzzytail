use clap::Parser;
use std::path::PathBuf;

mod config;
mod theme;
mod tail;
mod colorizer;
mod filter;
mod interactive;
mod output;
mod popup;

use config::Config;

#[derive(Parser)]
#[command(name = "ft")]
#[command(about = "A modern, colorful tail replacement with split-pane log monitoring")]
struct Cli {
    /// Files to tail
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Number of lines to show initially
    #[arg(short = 'n', long = "lines", default_value = "10")]
    lines: usize,

    /// Output the last NUM bytes instead of lines
    #[arg(short = 'c', long = "bytes")]
    bytes: Option<usize>,

    /// Never output headers giving file names
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Always output headers giving file names
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    /// Follow file changes (like tail -f). Auto-enabled for multiple files.
    #[arg(short = 'f', long = "follow")]
    follow: bool,

    /// Disable auto-follow when multiple files are given
    #[arg(long = "no-follow")]
    no_follow: bool,

    /// Config file path
    #[arg(long = "config")]
    config: Option<PathBuf>,

    /// Disable colors
    #[arg(long = "no-color")]
    no_color: bool,

    /// Include only lines matching this regex
    #[arg(long = "include")]
    include: Option<String>,

    /// Exclude lines matching this regex
    #[arg(long = "exclude")]
    exclude: Option<String>,

    /// Show only lines with specified log level (ERROR, WARN, INFO, DEBUG)
    #[arg(long = "level")]
    level: Option<String>,

    /// Interactive mode with keyboard controls
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,

    /// Output format: text (default), json, csv
    #[arg(long = "format", default_value = "text")]
    format: String,

    /// Buffer size for file operations (in bytes)
    #[arg(long = "buffer-size", default_value = "65536")]
    buffer_size: usize,

    /// Maximum lines to keep in scrollback buffer per window
    #[arg(long = "buffer-lines", default_value = "1000")]
    buffer_lines: usize,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    // Load configuration
    let config = Config::load(args.config.as_deref())?;

    // Auto-follow for multiple files (like multitail), unless --no-follow
    let follow = if args.no_follow {
        false
    } else if args.files.len() > 1 {
        true
    } else {
        args.follow
    };

    // Initialize tail processor
    let mut tail_processor = tail::TailProcessor::new(
        config,
        args.no_color,
        args.include,
        args.exclude,
        args.level,
        args.interactive,
        args.format,
        args.buffer_size,
        args.bytes,
        args.quiet,
        args.verbose,
        args.buffer_lines,
    )?;

    if args.files.is_empty() {
        use is_terminal::IsTerminal;
        if std::io::stdin().is_terminal() {
            tail_processor.show_default_logs(args.lines)?;
        } else {
            tail_processor.process_stdin(args.lines, follow)?;
        }
    } else {
        tail_processor.process_files(&args.files, args.lines, follow)?;
    }

    Ok(())
}
