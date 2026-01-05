//! Terminal User Interface using ratatui

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use crate::keyboard::{self, ActiveHand, Layer, SyncHidMonitor};

pub fn run(vil_path: PathBuf, use_hid: bool) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, vil_path, use_hid);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    vil_path: PathBuf,
    use_hid: bool,
) -> Result<()> {
    let mut layout_data = keyboard::load_layout(&vil_path)?;
    let mut current_layer: usize = 0;
    let mut file_mtime = std::fs::metadata(&vil_path)?.modified()?;

    // Setup file watcher
    let (tx, rx) = mpsc::channel();
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    })?;
    watcher.watch(&vil_path, RecursiveMode::NonRecursive)?;

    // Setup HID monitor if enabled
    let mut hid_monitor = if use_hid {
        match SyncHidMonitor::new() {
            Ok(monitor) => {
                log::info!("Connected to SZR35 keyboard");
                Some(monitor)
            }
            Err(e) => {
                log::warn!("HID not available: {}. Using interactive mode.", e);
                None
            }
        }
    } else {
        None
    };

    loop {
        // Check for file changes
        if let Ok(event) = rx.try_recv() {
            if matches!(event.kind, EventKind::Modify(_)) {
                if let Ok(new_mtime) = std::fs::metadata(&vil_path).and_then(|m| m.modified()) {
                    if new_mtime > file_mtime {
                        file_mtime = new_mtime;
                        if let Ok(new_layout) = keyboard::load_layout(&vil_path) {
                            layout_data = new_layout;
                        }
                    }
                }
            }
        }

        // Check HID for layer changes
        if let Some(ref mut monitor) = hid_monitor {
            if let Some(layer) = monitor.poll() {
                current_layer = layer as usize;
            }
        }

        // Draw UI
        terminal.draw(|f| draw_ui(f, &layout_data, current_layer))?;

        // Handle input with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let layer = c.to_digit(10).unwrap() as usize;
                        if layer < layout_data.len() {
                            current_layer = layer;
                        }
                    }
                    KeyCode::Char('n') | KeyCode::Char(' ') => {
                        current_layer = (current_layer + 1) % layout_data.len().min(8);
                    }
                    KeyCode::Char('p') => {
                        current_layer = (current_layer + layout_data.len().min(8) - 1)
                            % layout_data.len().min(8);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}

fn draw_ui(f: &mut Frame, layout_data: &[Layer], current_layer: usize) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(8),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(area);

    let layer_name = keyboard::layer_name(current_layer);
    let layer_color = str_to_color(keyboard::layer_color(current_layer));
    let title = Paragraph::new(layer_name)
        .style(Style::default().fg(layer_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    if current_layer < layout_data.len() {
        draw_keyboard(f, &layout_data[current_layer], current_layer, chunks[1]);
    }

    draw_layer_bar(f, current_layer, chunks[2]);

    let help = Paragraph::new("Keys: 0-7 = layers, n/p = next/prev, q = quit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    f.render_widget(help, chunks[3]);
}

fn draw_keyboard(f: &mut Frame, layer: &Layer, layer_idx: usize, area: Rect) {
    let active_hand = keyboard::active_hand(layer_idx);
    let row_height = 2;
    let keyboard_height = row_height * 4;
    let start_y = area.y + (area.height.saturating_sub(keyboard_height)) / 2;

    let left_rows = &layer[0..4];
    let right_rows = &layer[4..8];

    let key_width = 7u16;
    let gap = 4u16;
    let total_width = key_width * 10 + gap;
    let start_x = area.x + (area.width.saturating_sub(total_width)) / 2;

    for row_idx in 0..4 {
        let y = start_y + (row_idx as u16) * row_height;

        // Left hand
        let left = &left_rows[row_idx];
        for (col, kc) in left.iter().enumerate() {
            if kc.is_empty() && row_idx == 3 && col < 2 {
                continue;
            }

            let label = keyboard::simplify_keycode(kc);
            let color = if row_idx == 3 {
                str_to_color(keyboard::THUMB_COLOR)
            } else {
                str_to_color(keyboard::finger_color(col))
            };

            let dimmed = active_hand == ActiveHand::Right && !label.is_empty() && label != "·";
            let style = if dimmed || label == "·" {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            };

            let x = if row_idx == 3 {
                start_x + ((col as u16).saturating_sub(2) + 2) * key_width
            } else {
                start_x + (col as u16) * key_width
            };

            let key_area = Rect::new(x, y, key_width.saturating_sub(1), row_height);
            let key = Paragraph::new(label).style(style).alignment(Alignment::Center);
            f.render_widget(key, key_area);
        }

        // Right hand
        let right = &right_rows[row_idx];
        for (col, kc) in right.iter().enumerate() {
            if kc.is_empty() && row_idx == 3 && col >= 3 {
                continue;
            }

            let label = keyboard::simplify_keycode(kc);
            let color = if row_idx == 3 {
                str_to_color(keyboard::THUMB_COLOR)
            } else {
                str_to_color(keyboard::finger_color(col + 5))
            };

            let dimmed = active_hand == ActiveHand::Left && !label.is_empty() && label != "·";
            let style = if dimmed || label == "·" {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            };

            let x = start_x + 5 * key_width + gap + (col as u16) * key_width;
            let key_area = Rect::new(x, y, key_width.saturating_sub(1), row_height);
            let key = Paragraph::new(label).style(style).alignment(Alignment::Center);
            f.render_widget(key, key_area);
        }
    }
}

fn draw_layer_bar(f: &mut Frame, current_layer: usize, area: Rect) {
    let mut spans = Vec::new();

    for i in 0..8 {
        let name = keyboard::layer_name(i).split_whitespace().next().unwrap_or("?");
        let short = if name.len() > 4 { &name[..4] } else { name };

        if i == current_layer {
            let color = str_to_color(keyboard::layer_color(i));
            spans.push(Span::styled(
                format!(" {} ", short),
                Style::default()
                    .fg(Color::Black)
                    .bg(color)
                    .add_modifier(Modifier::BOLD),
            ));
        } else {
            spans.push(Span::styled(
                format!(" {} ", short),
                Style::default().fg(Color::DarkGray),
            ));
        }
        spans.push(Span::raw(" "));
    }

    let line = Line::from(spans);
    let bar = Paragraph::new(line).alignment(Alignment::Center);
    f.render_widget(bar, area);
}

fn str_to_color(s: &str) -> Color {
    match s {
        "cyan" => Color::Cyan,
        "magenta" => Color::Magenta,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "red" => Color::Red,
        "white" => Color::White,
        "light_yellow" => Color::LightYellow,
        _ => Color::White,
    }
}
