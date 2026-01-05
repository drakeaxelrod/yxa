//! Yxa Visual Guide
//!
//! A visual guide and trainer for the Yxa keyboard layout.
//! Defaults to GUI mode, use --tui for terminal interface.

mod keyboard;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(name = "yxa-visual-guide")]
#[command(about = "Yxa keyboard layout visual guide")]
#[command(version)]
struct Cli {
    /// Use terminal UI instead of GUI
    #[arg(long)]
    tui: bool,

    /// Path to .vil layout file
    #[arg(short, long)]
    file: Option<PathBuf>,

    /// Disable HID layer auto-detection
    #[arg(long)]
    no_hid: bool,

    /// Verbosity level (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Run in foreground (GUI normally detaches)
    #[arg(long)]
    foreground: bool,
}

fn find_layout_file(specified: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = specified {
        return Ok(path);
    }

    // Try relative to executable (handles installed location)
    if let Ok(exe) = std::env::current_exe() {
        let mut check = exe.parent().map(|p| p.to_path_buf());
        for _ in 0..5 {
            if let Some(ref dir) = check {
                let candidate = dir.join("layouts/miryoku-kbd-layout.vil");
                if candidate.exists() {
                    return Ok(candidate);
                }
                check = dir.parent().map(|p| p.to_path_buf());
            }
        }
    }

    // Try current directory
    let cwd = std::env::current_dir()?;
    let path = cwd.join("layouts/miryoku-kbd-layout.vil");
    if path.exists() {
        return Ok(path);
    }

    // Try config directory
    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("yxa/layout.vil");
        if path.exists() {
            return Ok(path);
        }
    }

    anyhow::bail!("Layout file not found. Use --file to specify path.")
}

fn init_logging(verbosity: u8) {
    let level = match verbosity {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::new()
        .filter_level(level)
        .format_timestamp(None)
        .init();
}

fn spawn_detached() -> Result<()> {
    let exe = std::env::current_exe()?;
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    args.push("--foreground".to_string());

    Command::new(&exe)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    init_logging(cli.verbose);

    let vil_path = find_layout_file(cli.file)?;
    let use_hid = !cli.no_hid;

    if cli.tui {
        // TUI mode - always runs in foreground
        ui::run_tui(vil_path, use_hid)?;
    } else {
        // GUI mode - detach unless --foreground
        if !cli.foreground {
            spawn_detached()?;
            return Ok(());
        }
        ui::run_gui(vil_path, use_hid)?;
    }

    Ok(())
}
