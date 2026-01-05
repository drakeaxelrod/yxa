//! Graphical User Interface using iced

use crate::keyboard::{load_layout, parse_key_label, HoldType, KeyLabel, Layer, SyncHidMonitor};
use anyhow::Result;
use iced::widget::{button, column, container, row, slider, text, Space};
use iced::window;
use iced::{event, keyboard, mouse, time, Background, Border, Color, Element, Event, Font, Length, Padding, Point, Subscription, Task, Theme};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

// Embedded Lilex Nerd Font
const LILEX_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/LilexNerdFont-Regular.ttf");
const LILEX_FONT: Font = Font::with_name("Lilex Nerd Font");

const KEY_SIZE: f32 = 50.0;
const KEY_GAP: f32 = 4.0;
const THUMB_GAP: f32 = 10.0;

// Column stagger offsets (pixels from top) - middle highest, pinky lowest
const COL_OFFSETS: [f32; 5] = [
    16.0,  // Pinky - lowest
    8.0,   // Ring - medium
    0.0,   // Middle - highest
    8.0,   // Index - medium
    16.0,  // Inner - lowest
];

const LAYER_INDICATOR_HEIGHT: f32 = 28.0;
const LAYER_NAMES: [&str; 9] = ["Base", "Nav", "Mouse", "Media", "Num", "Sym", "Fun", "Btn", "Extra"];

// Layer colors (RGB) using OneDark Pro palette
const LAYER_COLORS: [(f32, f32, f32); 9] = [
    (0.671, 0.698, 0.749), // Base - white (#abb2bf)
    (0.337, 0.714, 0.761), // Nav - cyan (#56b6c2)
    (0.596, 0.765, 0.475), // Mouse - green (#98c379)
    (0.776, 0.471, 0.867), // Media - purple (#c678dd)
    (0.898, 0.753, 0.482), // Num - yellow (#e5c07b)
    (0.878, 0.424, 0.459), // Sym - red (#e06c75)
    (0.380, 0.686, 0.937), // Fun - blue (#61afef)
    (0.361, 0.388, 0.439), // Button - bright black (#5c6370)
    (0.671, 0.698, 0.749), // Extra - white (#abb2bf)
];

// OneDark Pro base colors
const OD_BG: (f32, f32, f32) = (0.157, 0.173, 0.204);       // #282c34
const OD_FG: (f32, f32, f32) = (0.671, 0.698, 0.749);       // #abb2bf
const OD_BORDER: (f32, f32, f32) = (0.361, 0.388, 0.439);   // #5c6370

#[derive(Debug, Clone)]
pub struct Settings {
    pub bg_transparency: f32,
    pub key_transparency: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bg_transparency: 0.0,
            key_transparency: 1.0,
        }
    }
}

const WINDOW_WIDTH: f32 = 720.0;
const WINDOW_HEIGHT: f32 = 380.0;

pub fn run(vil_path: PathBuf, use_hid: bool) -> Result<()> {
    iced::application("Yxa Visual Guide", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| Theme::Dark)
        .font(LILEX_FONT_BYTES)
        .default_font(LILEX_FONT)
        .style(|_state, _theme| iced::daemon::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: Color::WHITE,
        })
        .window(window::Settings {
            size: iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            position: window::Position::Centered,
            decorations: false,
            resizable: false,
            transparent: true,
            ..Default::default()
        })
        .run_with(move || App::new(vil_path.clone(), use_hid))?;

    Ok(())
}

struct App {
    pressed_keys: HashSet<String>,
    current_layer: usize,
    settings: Settings,
    show_settings: bool,
    context_menu_position: Option<Point>,
    layout: Option<Vec<Layer>>,
    use_hid: bool,
    hid_monitor: Option<SyncHidMonitor>,
    shift_held: bool,
    ctrl_held: bool,
    alt_held: bool,
    gui_held: bool,
}

#[derive(Debug, Clone)]
enum Message {
    KeyPressed(keyboard::Key),
    KeyReleased(keyboard::Key),
    RightClick(Point),
    LeftClick(Point),
    CloseContextMenu,
    OpenSettings,
    CloseSettings,
    CloseApp,
    SetBgTransparency(f32),
    SetKeyTransparency(f32),
    DragWindow,
    HidTick,
    LayerChanged(usize),
}

impl App {
    fn new(vil_path: PathBuf, use_hid: bool) -> (Self, Task<Message>) {
        // Try to load layout
        let layout = load_layout(&vil_path).ok();

        // Try to connect to HID
        let hid_monitor = if use_hid {
            SyncHidMonitor::new().ok()
        } else {
            None
        };

        (Self {
            pressed_keys: HashSet::new(),
            current_layer: 0,
            settings: Settings::default(),
            show_settings: false,
            context_menu_position: None,
            layout,
            use_hid,
            hid_monitor,
            shift_held: false,
            ctrl_held: false,
            alt_held: false,
            gui_held: false,
        }, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::KeyPressed(key) => {
                // Close context menu on any key press
                self.context_menu_position = None;

                // Close settings on Escape
                if let keyboard::Key::Named(keyboard::key::Named::Escape) = &key {
                    if self.show_settings {
                        self.show_settings = false;
                        return Task::none();
                    }
                }

                // Track modifiers
                match &key {
                    keyboard::Key::Named(named) => {
                        match named {
                            keyboard::key::Named::Shift => self.shift_held = true,
                            keyboard::key::Named::Control => self.ctrl_held = true,
                            keyboard::key::Named::Alt => self.alt_held = true,
                            keyboard::key::Named::Super => self.gui_held = true,
                            _ => {}
                        }
                    }
                    _ => {}
                }

                match &key {
                    keyboard::Key::Character(c) => {
                        let key_char = c.to_string().to_uppercase();
                        self.pressed_keys.insert(key_char);
                    }
                    keyboard::Key::Named(named) => {
                        let name = match named {
                            keyboard::key::Named::Escape => Some("ESC"),
                            keyboard::key::Named::Space => Some("SPC"),
                            keyboard::key::Named::Tab => Some("TAB"),
                            keyboard::key::Named::Enter => Some("ENT"),
                            keyboard::key::Named::Backspace => Some("BSP"),
                            keyboard::key::Named::Delete => Some("DEL"),
                            keyboard::key::Named::ArrowLeft => Some("←"),
                            keyboard::key::Named::ArrowRight => Some("→"),
                            keyboard::key::Named::ArrowUp => Some("↑"),
                            keyboard::key::Named::ArrowDown => Some("↓"),
                            keyboard::key::Named::Home => Some("HOM"),
                            keyboard::key::Named::End => Some("END"),
                            keyboard::key::Named::PageUp => Some("PGU"),
                            keyboard::key::Named::PageDown => Some("PGD"),
                            _ => None,
                        };
                        if let Some(n) = name {
                            self.pressed_keys.insert(n.to_string());
                        }
                    }
                    _ => {}
                }
            }
            Message::KeyReleased(key) => {
                // Track modifiers
                match &key {
                    keyboard::Key::Named(named) => {
                        match named {
                            keyboard::key::Named::Shift => self.shift_held = false,
                            keyboard::key::Named::Control => self.ctrl_held = false,
                            keyboard::key::Named::Alt => self.alt_held = false,
                            keyboard::key::Named::Super => self.gui_held = false,
                            _ => {}
                        }
                    }
                    _ => {}
                }

                match &key {
                    keyboard::Key::Character(c) => {
                        let key_char = c.to_string().to_uppercase();
                        self.pressed_keys.remove(&key_char);
                    }
                    keyboard::Key::Named(named) => {
                        let name = match named {
                            keyboard::key::Named::Escape => Some("ESC"),
                            keyboard::key::Named::Space => Some("SPC"),
                            keyboard::key::Named::Tab => Some("TAB"),
                            keyboard::key::Named::Enter => Some("ENT"),
                            keyboard::key::Named::Backspace => Some("BSP"),
                            keyboard::key::Named::Delete => Some("DEL"),
                            keyboard::key::Named::ArrowLeft => Some("←"),
                            keyboard::key::Named::ArrowRight => Some("→"),
                            keyboard::key::Named::ArrowUp => Some("↑"),
                            keyboard::key::Named::ArrowDown => Some("↓"),
                            keyboard::key::Named::Home => Some("HOM"),
                            keyboard::key::Named::End => Some("END"),
                            keyboard::key::Named::PageUp => Some("PGU"),
                            keyboard::key::Named::PageDown => Some("PGD"),
                            _ => None,
                        };
                        if let Some(n) = name {
                            self.pressed_keys.remove(n);
                        }
                    }
                    _ => {}
                }
            }
            Message::RightClick(position) => {
                self.context_menu_position = Some(position);
            }
            Message::CloseContextMenu => {
                self.context_menu_position = None;
            }
            Message::OpenSettings => {
                self.context_menu_position = None;
                self.show_settings = true;
            }
            Message::CloseSettings => {
                self.show_settings = false;
            }
            Message::SetBgTransparency(value) => {
                self.settings.bg_transparency = value;
            }
            Message::SetKeyTransparency(value) => {
                self.settings.key_transparency = value;
            }
            Message::CloseApp => {
                return window::get_latest().and_then(window::close);
            }
            Message::LeftClick(_pos) => {
                self.context_menu_position = None;
            }
            Message::DragWindow => {
                if !self.show_settings {
                    return window::get_latest().and_then(|id| window::drag(id));
                }
            }
            Message::HidTick => {
                // Poll HID for layer changes
                if let Some(ref mut monitor) = self.hid_monitor {
                    if let Some(layer) = monitor.poll() {
                        self.current_layer = layer as usize;
                    }
                }
            }
            Message::LayerChanged(layer) => {
                self.current_layer = layer;
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let left_half = self.render_left_half();
        let right_half = self.render_right_half();

        // Build layer indicators with colors
        let mut layer_elements: Vec<Element<Message>> = Vec::new();
        for (i, name) in LAYER_NAMES.iter().enumerate() {
            let is_active = i == self.current_layer;
            layer_elements.push(self.render_layer_indicator(name, i, is_active));
            if i < 8 {
                layer_elements.push(Space::with_width(4.0).into());
            }
        }
        let layer_row = row(layer_elements);

        let centered_layers = container(layer_row)
            .width(Length::Fill)
            .center_x(Length::Fill);

        let logo = self.render_logo();

        let keyboard = row![
            left_half,
            Space::with_width(Length::Fill),
            logo,
            Space::with_width(Length::Fill),
            right_half,
        ];

        let content = column![
            keyboard,
            Space::with_height(30.0),
            centered_layers,
        ];

        let bg_alpha = self.settings.bg_transparency;

        // Inner container with rounded corners and background
        let inner_container = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(20)
            .style(move |_| container::Style {
                background: Some(Background::Color(Color::from_rgba(0.1, 0.1, 0.1, bg_alpha))),
                border: Border {
                    radius: 12.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            });

        // Outer container with fully transparent background
        let main_view: Element<'_, Message> = container(inner_container)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(4)
            .style(|_| container::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                ..Default::default()
            })
            .into();

        // Stack overlays
        if self.show_settings {
            self.render_settings_overlay(main_view)
        } else if let Some(pos) = self.context_menu_position {
            self.render_context_menu(main_view, pos)
        } else {
            main_view
        }
    }

    /// Get key label from layout for given position
    fn get_key_label(&self, hand: usize, row: usize, col: usize) -> KeyLabel {
        if let Some(ref layout) = self.layout {
            if self.current_layer < layout.len() {
                let layer = &layout[self.current_layer];
                // Layout is: rows 0-3 left, rows 4-7 right
                let layout_row = if hand == 0 { row } else { row + 4 };
                if layout_row < layer.len() && col < layer[layout_row].len() {
                    let keycode = &layer[layout_row][col];
                    let mut label = parse_key_label(keycode);

                    // Apply shift transformation
                    if label.tap.len() == 1 {
                        let c = label.tap.chars().next().unwrap();
                        if c.is_ascii_alphabetic() {
                            // Show lowercase normally, uppercase when shift held
                            label.tap = if self.shift_held {
                                c.to_ascii_uppercase().to_string()
                            } else {
                                c.to_ascii_lowercase().to_string()
                            };
                        } else if self.shift_held {
                            // Show shifted symbols
                            label.tap = match c {
                                '1' => "!".to_string(),
                                '2' => "@".to_string(),
                                '3' => "#".to_string(),
                                '4' => "$".to_string(),
                                '5' => "%".to_string(),
                                '6' => "^".to_string(),
                                '7' => "&".to_string(),
                                '8' => "*".to_string(),
                                '9' => "(".to_string(),
                                '0' => ")".to_string(),
                                '-' => "_".to_string(),
                                '=' => "+".to_string(),
                                '[' => "{".to_string(),
                                ']' => "}".to_string(),
                                '\\' => "|".to_string(),
                                ';' => ":".to_string(),
                                '\'' => "\"".to_string(),
                                ',' => "<".to_string(),
                                '.' => ">".to_string(),
                                '/' => "?".to_string(),
                                '`' => "~".to_string(),
                                _ => label.tap,
                            };
                        }
                    }
                    return label;
                }
            }
        }
        KeyLabel { tap: "".to_string(), hold: None }
    }

    /// Check if a key is currently pressed based on its tap label
    fn is_key_pressed(&self, tap: &str) -> bool {
        if tap.is_empty() || tap == "󰝦" || tap == "▽" || tap == "·" {
            return false;
        }
        // Check single character keys
        if tap.len() == 1 {
            return self.pressed_keys.contains(&tap.to_uppercase());
        }
        // Check special keys
        self.pressed_keys.contains(tap)
    }

    fn render_left_half(&self) -> Element<'_, Message> {
        let mut columns: Vec<Element<Message>> = Vec::new();

        for col in 0..5 {
            let offset = COL_OFFSETS[col];
            let mut col_elements: Vec<Element<Message>> = Vec::new();

            if offset > 0.0 {
                col_elements.push(Space::with_height(offset).into());
            }

            for row_idx in 0..3 {
                let label = self.get_key_label(0, row_idx, col);
                let pressed = self.is_key_pressed(&label.tap);
                col_elements.push(self.render_key(label, pressed, col));
                if row_idx < 2 {
                    col_elements.push(Space::with_height(KEY_GAP).into());
                }
            }

            let bottom_pad = 16.0 - offset;
            if bottom_pad > 0.0 {
                col_elements.push(Space::with_height(bottom_pad).into());
            }

            columns.push(column(col_elements).into());
            if col < 4 {
                columns.push(Space::with_width(KEY_GAP).into());
            }
        }

        let finger_keys = row(columns);

        // Thumb cluster
        let thumb_swoop: [f32; 3] = [0.0, 4.0, 8.0];
        let mut thumb_elements: Vec<Element<Message>> = Vec::new();

        // Thumb row is row 3 (index 3), columns 2,3,4 contain actual keys
        for i in 0..3 {
            let offset = thumb_swoop[i];
            let label = self.get_key_label(0, 3, i + 2); // columns 2,3,4
            let pressed = self.is_key_pressed(&label.tap);
            let thumb_col = column![
                Space::with_height(offset),
                self.render_thumb_key(label, pressed),
            ];
            thumb_elements.push(thumb_col.into());
            if i < 2 {
                thumb_elements.push(Space::with_width(THUMB_GAP).into());
            }
        }
        let thumb_cluster = row(thumb_elements);

        let thumb_width = 3.0 * KEY_SIZE + 2.0 * THUMB_GAP;
        let finger_width = 5.0 * KEY_SIZE + 4.0 * KEY_GAP;
        let extra_shift = KEY_SIZE / 3.0;
        let thumb_row = row![
            Space::with_width(finger_width - thumb_width + extra_shift),
            thumb_cluster,
        ];

        column![
            finger_keys,
            Space::with_height(10.0),
            thumb_row,
        ].into()
    }

    fn render_right_half(&self) -> Element<'_, Message> {
        let mut columns: Vec<Element<Message>> = Vec::new();

        // Right hand: col 0 = inner (display left), col 4 = pinky (display right)
        // Offsets are mirrored: inner gets pinky offset, pinky gets pinky offset
        let right_offsets: [f32; 5] = [
            16.0,  // Inner (col 0) - lowest
            8.0,   // Index (col 1) - medium
            0.0,   // Middle (col 2) - highest
            8.0,   // Ring (col 3) - medium
            16.0,  // Pinky (col 4) - lowest
        ];

        for col in 0..5 {
            let offset = right_offsets[col];
            let mut col_elements: Vec<Element<Message>> = Vec::new();

            if offset > 0.0 {
                col_elements.push(Space::with_height(offset).into());
            }

            for row_idx in 0..3 {
                let label = self.get_key_label(1, row_idx, col);
                let pressed = self.is_key_pressed(&label.tap);
                // Use column index for color (5-9 for right hand, mirrored)
                col_elements.push(self.render_key(label, pressed, 9 - col));
                if row_idx < 2 {
                    col_elements.push(Space::with_height(KEY_GAP).into());
                }
            }

            let bottom_pad = 16.0 - offset;
            if bottom_pad > 0.0 {
                col_elements.push(Space::with_height(bottom_pad).into());
            }

            columns.push(column(col_elements).into());
            if col < 4 {
                columns.push(Space::with_width(KEY_GAP).into());
            }
        }

        let finger_keys = row(columns);

        // Thumb cluster - mirrored
        let thumb_swoop: [f32; 3] = [8.0, 4.0, 0.0];
        let mut thumb_elements: Vec<Element<Message>> = Vec::new();

        // Right thumb row is row 3 (hand 1), columns 0,1,2 contain actual keys
        for i in 0..3 {
            let offset = thumb_swoop[i];
            let label = self.get_key_label(1, 3, i);
            let pressed = self.is_key_pressed(&label.tap);
            let thumb_col = column![
                Space::with_height(offset),
                self.render_thumb_key(label, pressed),
            ];
            thumb_elements.push(thumb_col.into());
            if i < 2 {
                thumb_elements.push(Space::with_width(THUMB_GAP).into());
            }
        }
        let thumb_cluster = row(thumb_elements);

        let extra_shift = KEY_SIZE / 3.0;
        let finger_row = row![
            Space::with_width(extra_shift),
            finger_keys,
        ];

        column![
            finger_row,
            Space::with_height(10.0),
            thumb_cluster,
        ].into()
    }

    fn render_key(&self, label: KeyLabel, pressed: bool, col: usize) -> Element<'static, Message> {
        use iced::widget::stack;

        let alpha = self.settings.key_transparency;
        let layer_color = LAYER_COLORS[self.current_layer];

        let bg_color = if pressed {
            Color::from_rgba(layer_color.0, layer_color.1, layer_color.2, alpha * 0.8)
        } else {
            Color::from_rgba(OD_BG.0, OD_BG.1, OD_BG.2, alpha)
        };

        let border_color = if pressed {
            Color::from_rgba(layer_color.0, layer_color.1, layer_color.2, alpha)
        } else {
            // Subtle finger coloring on border using OneDark Pro palette
            let finger_colors: [(f32, f32, f32); 10] = [
                (0.337, 0.714, 0.761), // pinky - cyan (#56b6c2)
                (0.776, 0.471, 0.867), // ring - purple (#c678dd)
                (0.596, 0.765, 0.475), // middle - green (#98c379)
                (0.898, 0.753, 0.482), // index - yellow (#e5c07b)
                (0.898, 0.753, 0.482), // inner - yellow
                (0.898, 0.753, 0.482), // inner - yellow
                (0.898, 0.753, 0.482), // index - yellow
                (0.596, 0.765, 0.475), // middle - green
                (0.776, 0.471, 0.867), // ring - purple
                (0.337, 0.714, 0.761), // pinky - cyan
            ];
            let fc = finger_colors.get(col).unwrap_or(&OD_BORDER);
            Color::from_rgba(fc.0, fc.1, fc.2, alpha * 0.6)
        };

        let tap = label.tap;
        let is_empty_key = tap == "▽" || tap == "·";

        let text_color = if pressed {
            Color { a: alpha, ..Color::WHITE }
        } else if is_empty_key {
            // Fade empty/transparent keys
            Color::from_rgba(OD_FG.0, OD_FG.1, OD_FG.2, alpha * 0.3)
        } else {
            Color::from_rgba(OD_FG.0, OD_FG.1, OD_FG.2, alpha)
        };

        let tap_font_size = if tap.len() > 3 { 10.0 } else { 14.0 };

        // Tap label centered
        let tap_text = container(text(tap.clone()).size(tap_font_size).color(text_color))
            .width(KEY_SIZE)
            .height(KEY_SIZE)
            .center_x(KEY_SIZE)
            .center_y(KEY_SIZE);

        // Hold label in bottom-right corner (if present)
        if let Some(hold) = label.hold {
            let (hold_label, hold_color) = match hold {
                HoldType::Modifier(m) => {
                    (m, Color::from_rgba(0.6, 0.6, 0.6, alpha * 0.9))
                }
                HoldType::Layer(idx, name) => {
                    let lc = LAYER_COLORS.get(idx).unwrap_or(&(0.5, 0.5, 0.5));
                    (name, Color::from_rgba(lc.0, lc.1, lc.2, alpha))
                }
            };

            let hold_text = container(
                container(text(hold_label).size(9.0).color(hold_color))
                    .padding(Padding { top: 0.0, right: 4.0, bottom: 3.0, left: 0.0 })
            )
            .width(KEY_SIZE)
            .height(KEY_SIZE)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Bottom);

            container(stack![tap_text, hold_text])
                .width(KEY_SIZE)
                .height(KEY_SIZE)
                .style(move |_| container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: border_color,
                        width: 2.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            container(tap_text)
                .width(KEY_SIZE)
                .height(KEY_SIZE)
                .style(move |_| container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: border_color,
                        width: 2.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        }
    }

    fn render_thumb_key(&self, label: KeyLabel, pressed: bool) -> Element<'static, Message> {
        use iced::widget::stack;

        let alpha = self.settings.key_transparency;
        let layer_color = LAYER_COLORS[self.current_layer];

        let bg_color = if pressed {
            Color::from_rgba(layer_color.0, layer_color.1, layer_color.2, alpha * 0.8)
        } else {
            Color::from_rgba(OD_BG.0, OD_BG.1, OD_BG.2, alpha)
        };

        // Thumb keys get blue border (OneDark Pro blue)
        let border_color = if pressed {
            Color::from_rgba(layer_color.0, layer_color.1, layer_color.2, alpha)
        } else {
            Color::from_rgba(0.380, 0.686, 0.937, alpha * 0.6) // #61afef
        };

        let tap = label.tap;
        let is_empty_key = tap == "▽" || tap == "·";

        let text_color = if pressed {
            Color { a: alpha, ..Color::WHITE }
        } else if is_empty_key {
            // Fade empty/transparent keys
            Color::from_rgba(OD_FG.0, OD_FG.1, OD_FG.2, alpha * 0.3)
        } else {
            Color::from_rgba(OD_FG.0, OD_FG.1, OD_FG.2, alpha)
        };

        let tap_font_size = if tap.len() > 4 { 9.0 } else if tap.len() > 3 { 10.0 } else { 12.0 };

        // Tap label centered
        let tap_text = container(text(tap.clone()).size(tap_font_size).color(text_color))
            .width(KEY_SIZE)
            .height(KEY_SIZE)
            .center_x(KEY_SIZE)
            .center_y(KEY_SIZE);

        // Hold label in bottom-right corner (if present)
        if let Some(hold) = label.hold {
            let (hold_label, hold_color) = match hold {
                HoldType::Modifier(m) => {
                    (m, Color::from_rgba(0.6, 0.6, 0.6, alpha * 0.9))
                }
                HoldType::Layer(idx, name) => {
                    let lc = LAYER_COLORS.get(idx).unwrap_or(&(0.5, 0.5, 0.5));
                    (name, Color::from_rgba(lc.0, lc.1, lc.2, alpha))
                }
            };

            let hold_text = container(
                container(text(hold_label).size(9.0).color(hold_color))
                    .padding(Padding { top: 0.0, right: 4.0, bottom: 3.0, left: 0.0 })
            )
            .width(KEY_SIZE)
            .height(KEY_SIZE)
            .align_x(iced::alignment::Horizontal::Right)
            .align_y(iced::alignment::Vertical::Bottom);

            container(stack![tap_text, hold_text])
                .width(KEY_SIZE)
                .height(KEY_SIZE)
                .style(move |_| container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: border_color,
                        width: 2.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        } else {
            container(tap_text)
                .width(KEY_SIZE)
                .height(KEY_SIZE)
                .style(move |_| container::Style {
                    background: Some(Background::Color(bg_color)),
                    border: Border {
                        color: border_color,
                        width: 2.0,
                        radius: 6.0.into(),
                    },
                    ..Default::default()
                })
                .into()
        }
    }

    fn render_layer_indicator(&self, label: &str, layer_idx: usize, is_active: bool) -> Element<'static, Message> {
        let layer_color = LAYER_COLORS[layer_idx];

        // Base layer (idx 0) uses lighter bg when active for better contrast
        let bg_color = if is_active {
            if layer_idx == 0 {
                Color::from_rgb(0.5, 0.5, 0.5) // Lighter gray for base layer
            } else {
                Color::from_rgb(layer_color.0, layer_color.1, layer_color.2)
            }
        } else {
            Color::from_rgb(OD_BG.0, OD_BG.1, OD_BG.2)
        };

        let border_color = if is_active {
            Color::from_rgb(
                (layer_color.0 + 0.2).min(1.0),
                (layer_color.1 + 0.2).min(1.0),
                (layer_color.2 + 0.2).min(1.0),
            )
        } else {
            Color::from_rgb(layer_color.0 * 0.5, layer_color.1 * 0.5, layer_color.2 * 0.5)
        };

        // Use dark text for bright backgrounds, white for dark
        let text_color = if is_active {
            // Calculate luminance to determine text contrast
            let luminance = 0.299 * layer_color.0 + 0.587 * layer_color.1 + 0.114 * layer_color.2;
            if luminance > 0.5 || layer_idx == 0 {
                Color::from_rgb(0.1, 0.1, 0.1) // Dark text for bright backgrounds
            } else {
                Color::WHITE
            }
        } else {
            Color::from_rgb(0.5, 0.5, 0.5)
        };

        let label = label.to_string();

        // Fixed width for uniform sizing
        container(text(label).size(11).color(text_color))
            .width(50)
            .height(LAYER_INDICATOR_HEIGHT)
            .center_x(50)
            .center_y(LAYER_INDICATOR_HEIGHT)
            .style(move |_| container::Style {
                background: Some(Background::Color(bg_color)),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 4.0.into(),
                },
                ..Default::default()
            })
            .into()
    }

    fn render_logo(&self) -> Element<'_, Message> {
        // OneDark Pro colors for the logo
        let cyan = Color::from_rgb(0.337, 0.714, 0.761);    // #56b6c2
        let purple = Color::from_rgb(0.776, 0.471, 0.867);  // #c678dd
        let green = Color::from_rgb(0.596, 0.765, 0.475);   // #98c379

        let alpha = self.settings.key_transparency;

        // "Yxa" in cyan (brand name)
        let yxa = text("Yxa")
            .size(18)
            .color(Color::from_rgba(cyan.r, cyan.g, cyan.b, alpha));

        // "Visual" in purple
        let visual = text("Visual")
            .size(11)
            .color(Color::from_rgba(purple.r, purple.g, purple.b, alpha * 0.9));

        // "Guide" in green
        let guide = text("Guide")
            .size(11)
            .color(Color::from_rgba(green.r, green.g, green.b, alpha * 0.9));

        let logo_content = column![
            container(yxa).center_x(Length::Fill),
            container(visual).center_x(Length::Fill),
            container(guide).center_x(Length::Fill),
        ]
        .spacing(2);

        container(logo_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn render_context_menu<'a>(&self, base: Element<'a, Message>, _pos: Point) -> Element<'a, Message> {
        use iced::widget::stack;

        // OneDark Pro colors
        let bg_color = Color::from_rgb(0.157, 0.173, 0.204);      // #282c34
        let hover_color = Color::from_rgb(0.361, 0.388, 0.439);   // #5c6370
        let text_color = Color::from_rgb(0.671, 0.698, 0.749);    // #abb2bf
        let border_color = Color::from_rgb(0.361, 0.388, 0.439);  // #5c6370
        let red_color = Color::from_rgb(0.878, 0.424, 0.459);     // #e06c75

        let settings_btn = button(
            container(text("Settings").size(14).color(text_color))
                .width(Length::Fill)
                .padding([8, 16])
        )
        .width(Length::Fill)
        .on_press(Message::OpenSettings)
        .style(move |_, status| {
            let bg = match status {
                button::Status::Hovered => hover_color,
                _ => bg_color,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color,
                border: Border::default(),
                ..Default::default()
            }
        });

        let close_btn = button(
            container(text("Close").size(14).color(text_color))
                .width(Length::Fill)
                .padding([8, 16])
        )
        .width(Length::Fill)
        .on_press(Message::CloseApp)
        .style(move |_, status| {
            let bg = match status {
                button::Status::Hovered => red_color,
                _ => bg_color,
            };
            button::Style {
                background: Some(Background::Color(bg)),
                text_color,
                border: Border::default(),
                ..Default::default()
            }
        });

        let menu = container(
            column![
                settings_btn,
                close_btn,
            ].width(120)
        )
        .style(move |_| container::Style {
            background: Some(Background::Color(bg_color)),
            border: Border {
                color: border_color,
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

        let backdrop = button(Space::new(Length::Fill, Length::Fill))
            .on_press(Message::CloseContextMenu)
            .style(|_, _| button::Style {
                background: Some(Background::Color(Color::TRANSPARENT)),
                ..Default::default()
            });

        let menu_positioned = container(menu)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        stack![
            base,
            backdrop,
            menu_positioned,
        ].into()
    }

    fn render_settings_overlay<'a>(&self, base: Element<'a, Message>) -> Element<'a, Message> {
        use iced::widget::stack;

        // OneDark Pro colors
        let bg_color = Color::from_rgb(0.157, 0.173, 0.204);      // #282c34
        let text_color = Color::from_rgb(0.671, 0.698, 0.749);    // #abb2bf
        let border_color = Color::from_rgb(0.361, 0.388, 0.439);  // #5c6370
        let red_color = Color::from_rgb(0.878, 0.424, 0.459);     // #e06c75
        let green_color = Color::from_rgb(0.596, 0.765, 0.475);   // #98c379

        let title = text("Settings").size(20).color(text_color);

        let close_btn = button(text("X").size(16).color(text_color))
            .on_press(Message::CloseSettings)
            .style(move |_, status| {
                let bg = match status {
                    button::Status::Hovered => red_color,
                    _ => border_color,
                };
                button::Style {
                    background: Some(Background::Color(bg)),
                    text_color,
                    border: Border {
                        radius: 4.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            });

        let header = row![
            title,
            Space::with_width(Length::Fill),
            close_btn,
        ].padding(10);

        let bg_trans_label = text("Background opacity:").size(14).color(text_color);
        let bg_trans_slider = slider(0.0..=1.0, self.settings.bg_transparency, Message::SetBgTransparency)
            .step(0.01)
            .width(150);
        let bg_trans_value = text(format!("{:.0}%", self.settings.bg_transparency * 100.0))
            .size(14)
            .color(text_color);

        let bg_trans_row = row![
            bg_trans_label,
            Space::with_width(20),
            bg_trans_slider,
            Space::with_width(10),
            bg_trans_value,
        ].align_y(iced::Alignment::Center);

        let key_trans_label = text("Key opacity:").size(14).color(text_color);
        let key_trans_slider = slider(0.0..=1.0, self.settings.key_transparency, Message::SetKeyTransparency)
            .step(0.01)
            .width(150);
        let key_trans_value = text(format!("{:.0}%", self.settings.key_transparency * 100.0))
            .size(14)
            .color(text_color);

        let key_trans_row = row![
            key_trans_label,
            Space::with_width(20),
            key_trans_slider,
            Space::with_width(10),
            key_trans_value,
        ].align_y(iced::Alignment::Center);

        // HID status with OneDark colors
        let hid_status = if self.hid_monitor.is_some() {
            text("HID: Connected").size(12).color(green_color)
        } else {
            text("HID: Not connected").size(12).color(red_color)
        };

        let settings_content = column![
            header,
            container(
                column![
                    bg_trans_row,
                    Space::with_height(15),
                    key_trans_row,
                    Space::with_height(15),
                    hid_status,
                ].padding(20)
            ),
        ];

        let settings_panel = container(settings_content)
            .width(400)
            .style(move |_| container::Style {
                background: Some(Background::Color(bg_color)),
                border: Border {
                    color: border_color,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            });

        let backdrop = button(Space::new(Length::Fill, Length::Fill))
            .on_press(Message::CloseSettings)
            .style(|_, _| button::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.5))),
                ..Default::default()
            });

        let centered_panel = container(settings_panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .padding(20);

        stack![
            base,
            backdrop,
            centered_panel,
        ].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            keyboard::on_key_press(|key, _mods| Some(Message::KeyPressed(key))),
            keyboard::on_key_release(|key, _mods| Some(Message::KeyReleased(key))),
            event::listen_with(|event, status, _window| {
                match event {
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                        Some(Message::RightClick(Point::ORIGIN))
                    }
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                        if matches!(status, event::Status::Ignored) {
                            Some(Message::DragWindow)
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            }),
        ];

        // Add HID polling if connected
        if self.hid_monitor.is_some() {
            subs.push(time::every(Duration::from_millis(50)).map(|_| Message::HidTick));
        }

        Subscription::batch(subs)
    }
}
