//! Graphical User Interface using iced

use crate::keyboard::{load_layout, parse_key_label, HidEvent, HoldType, KeyLabel, Layer, SyncHidMonitor};
use anyhow::Result;
use iced::widget::{button, checkbox, column, container, row, slider, text, Space};
use iced::window;
use iced::{event, keyboard, mouse, time, Background, Border, Color, Element, Event, Font, Length, Padding, Point, Subscription, Task, Theme};
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

// Embedded Lilex Nerd Font
const LILEX_FONT_BYTES: &[u8] = include_bytes!("../../assets/LilexNerdFont-Regular.ttf");
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
// Official Miryoku layer order: BASE, EXTRA, TAP, BUTTON, NAV, MOUSE, MEDIA, NUM, SYM, FUN
const LAYER_NAMES: [&str; 10] = ["Base", "Extra", "Tap", "Button", "Navigation", "Mouse", "Media", "Number", "Symbol", "Function"];

// OneDark Pro base colors
const OD_BG: (f32, f32, f32) = (0.157, 0.173, 0.204);       // #282c34
const OD_FG: (f32, f32, f32) = (0.671, 0.698, 0.749);       // #abb2bf
const OD_BORDER: (f32, f32, f32) = (0.361, 0.388, 0.439);   // #5c6370

// Layer colors (RGB) using OneDark Pro palette - matching official Miryoku order
const LAYER_COLORS: [(f32, f32, f32); 10] = [
    OD_FG,                 // 0 BASE - foreground (#abb2bf)
    OD_FG,                 // 1 EXTRA - foreground (#abb2bf)
    OD_FG,                 // 2 TAP - foreground (#abb2bf)
    OD_BORDER,             // 3 BUTTON - border/comment (#5c6370)
    (0.337, 0.714, 0.761), // 4 NAV - cyan (#56b6c2)
    (0.898, 0.753, 0.482), // 5 MOUSE - yellow (#e5c07b)
    (0.776, 0.471, 0.867), // 6 MEDIA - purple (#c678dd)
    (0.380, 0.686, 0.937), // 7 NUM - blue (#61afef)
    (0.596, 0.765, 0.475), // 8 SYM - green (#98c379)
    (0.878, 0.424, 0.459), // 9 FUN - red (#e06c75)
];

#[derive(Debug, Clone)]
pub struct Settings {
    pub bg_transparency: f32,
    pub key_transparency: f32,
    pub show_title: bool,
    pub show_layer_indicators: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bg_transparency: 0.0,
            key_transparency: 0.5,
            show_title: false,
            show_layer_indicators: false,
        }
    }
}

const WINDOW_WIDTH: f32 = 740.0;
const WINDOW_HEIGHT_FULL: f32 = 380.0;
const WINDOW_HEIGHT_NO_LAYERS: f32 = 320.0;
const LAYER_INDICATOR_SECTION_HEIGHT: f32 = 60.0;

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
            size: iced::Size::new(WINDOW_WIDTH, WINDOW_HEIGHT_NO_LAYERS),
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
    /// Pressed keys tracked by matrix position (row, col)
    pressed_keys: HashSet<(u8, u8)>,
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
    ToggleShowTitle(bool),
    ToggleShowLayerIndicators(bool),
    ResizeWindow,
    DragWindow,
    HidTick,
    LayerChanged(usize),
}

impl App {
    fn new(vil_path: PathBuf, use_hid: bool) -> (Self, Task<Message>) {
        // Try to load layout
        let layout = load_layout(&vil_path).ok();

        // Always create HID monitor if use_hid is true - it handles reconnection internally
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
                    }
                }
            }
            Message::KeyReleased(_) => {
                // Keypress highlighting is now handled via HID
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
            Message::ToggleShowTitle(value) => {
                self.settings.show_title = value;
            }
            Message::ToggleShowLayerIndicators(value) => {
                self.settings.show_layer_indicators = value;
                return Task::done(Message::ResizeWindow);
            }
            Message::ResizeWindow => {
                let height = if self.settings.show_layer_indicators {
                    WINDOW_HEIGHT_FULL
                } else {
                    WINDOW_HEIGHT_NO_LAYERS
                };
                return window::get_latest().and_then(move |id| {
                    window::resize(id, iced::Size::new(WINDOW_WIDTH, height))
                });
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
                // Poll HID for all buffered events at once (more efficient, catches rapid keypresses)
                let events = if let Some(ref mut monitor) = self.hid_monitor {
                    monitor.poll_all_events()
                } else {
                    Vec::new()
                };

                // Process all collected events
                for event in events {
                    match event {
                        HidEvent::LayerChange(layer) => {
                            self.current_layer = layer as usize;
                        }
                        HidEvent::KeyPress(key_event) => {
                            self.pressed_keys.insert((key_event.row, key_event.col));
                            self.update_modifier_state(key_event.row, key_event.col, true);
                        }
                        HidEvent::KeyRelease(key_event) => {
                            self.pressed_keys.remove(&(key_event.row, key_event.col));
                            self.update_modifier_state(key_event.row, key_event.col, false);
                        }
                        HidEvent::CapsWordState(active) => {
                            // Could add visual indicator for Caps Word
                            let _ = active; // TODO: Add caps word indicator to UI
                        }
                        HidEvent::ModifierState(mods) => {
                            // Update modifier state from firmware
                            self.shift_held = (mods & 0x02) != 0 || (mods & 0x20) != 0;
                            self.ctrl_held = (mods & 0x01) != 0 || (mods & 0x10) != 0;
                            self.alt_held = (mods & 0x04) != 0 || (mods & 0x40) != 0;
                            self.gui_held = (mods & 0x08) != 0 || (mods & 0x80) != 0;
                        }
                        HidEvent::FullState { layer, caps_word: _, modifiers, pressed_keys } => {
                            // Full state sync from firmware
                            self.current_layer = layer as usize;
                            self.shift_held = (modifiers & 0x02) != 0 || (modifiers & 0x20) != 0;
                            self.ctrl_held = (modifiers & 0x01) != 0 || (modifiers & 0x10) != 0;
                            self.alt_held = (modifiers & 0x04) != 0 || (modifiers & 0x40) != 0;
                            self.gui_held = (modifiers & 0x08) != 0 || (modifiers & 0x80) != 0;
                            self.pressed_keys.clear();
                            for (row, col) in pressed_keys {
                                self.pressed_keys.insert((row, col));
                            }
                        }
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

        // Build layer indicators with colors (all 10 layers)
        let mut layer_elements: Vec<Element<Message>> = Vec::new();
        for (i, name) in LAYER_NAMES.iter().enumerate() {
            let is_active = i == self.current_layer;
            layer_elements.push(self.render_layer_indicator(name, i, is_active));
            if i < 9 {
                layer_elements.push(Space::with_width(4.0).into());
            }
        }
        let layer_row = row(layer_elements);

        let centered_layers = container(layer_row)
            .width(Length::Fill)
            .center_x(Length::Fill);

        let keyboard = if self.settings.show_title {
            let logo = self.render_logo();
            row![
                left_half,
                Space::with_width(Length::Fill),
                logo,
                Space::with_width(Length::Fill),
                right_half,
            ]
        } else {
            row![
                Space::with_width(Length::Fill),
                left_half,
                Space::with_width(60.0),
                right_half,
                Space::with_width(Length::Fill),
            ]
        };

        let content = if self.settings.show_layer_indicators {
            column![
                keyboard,
                Space::with_height(30.0),
                centered_layers,
            ]
        } else {
            column![
                keyboard,
            ]
        };

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

    /// Get layer-tap hold info from base layer for a given position
    /// Used to show layer name when a layer key is being held
    fn get_base_layer_hold(&self, hand: usize, row: usize, col: usize) -> Option<HoldType> {
        if let Some(ref layout) = self.layout {
            if !layout.is_empty() {
                let base_layer = &layout[0]; // Layer 0 = BASE
                let layout_row = if hand == 0 { row } else { row + 4 };
                if layout_row < base_layer.len() && col < base_layer[layout_row].len() {
                    let keycode = &base_layer[layout_row][col];
                    let label = parse_key_label(keycode);
                    return label.hold;
                }
            }
        }
        None
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

    /// Check if a key at the given matrix position is currently pressed
    fn is_key_pressed_at(&self, hand: usize, row: usize, col: usize) -> bool {
        // Convert visual position to matrix position
        // Left hand: rows 0-3, Right hand: rows 4-7
        let matrix_row = if hand == 0 { row } else { row + 4 };
        self.pressed_keys.contains(&(matrix_row as u8, col as u8))
    }

    /// Update modifier state based on key position
    /// In Miryoku, home row mods are on the base layer:
    /// Left home row (row 1): A=GUI, R=Alt, S=Ctrl, T=Shift
    /// Right home row (row 1): N=Shift, E=Ctrl, I=Alt, O=GUI
    fn update_modifier_state(&mut self, row: u8, col: u8, pressed: bool) {
        // Only check home row for modifiers (row 1 on left, row 5 on right)
        match (row, col) {
            // Left hand home row mods
            (1, 0) => self.gui_held = pressed,   // A = GUI
            (1, 1) => self.alt_held = pressed,   // R = Alt
            (1, 2) => self.ctrl_held = pressed,  // S = Ctrl
            (1, 3) => self.shift_held = pressed, // T = Shift
            // Right hand home row mods
            (5, 1) => self.shift_held = pressed, // N = Shift
            (5, 2) => self.ctrl_held = pressed,  // E = Ctrl
            (5, 3) => self.alt_held = pressed,   // I = Alt
            (5, 4) => self.gui_held = pressed,   // O = GUI
            _ => {}
        }
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
                let pressed = self.is_key_pressed_at(0, row_idx, col);
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
            let col = i + 2;
            let label = self.get_key_label(0, 3, col);
            let pressed = self.is_key_pressed_at(0, 3, col);
            // Get base layer hold info for layer name when pressed
            let base_hold = if pressed { self.get_base_layer_hold(0, 3, col) } else { None };
            let thumb_col = column![
                Space::with_height(offset),
                self.render_thumb_key(label, pressed, base_hold),
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
                let pressed = self.is_key_pressed_at(1, row_idx, col);
                // Use column index for color (5-9 for right hand: inner=5, index=6, middle=7, ring=8, pinky=9)
                col_elements.push(self.render_key(label, pressed, col + 5));
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
            let pressed = self.is_key_pressed_at(1, 3, i);
            // Get base layer hold info for layer name when pressed
            let base_hold = if pressed { self.get_base_layer_hold(1, 3, i) } else { None };
            let thumb_col = column![
                Space::with_height(offset),
                self.render_thumb_key(label, pressed, base_hold),
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
        let is_empty_key = tap.is_empty() || tap == " " || tap == "▽" || tap == "·";

        let text_color = if pressed {
            Color { a: alpha, ..Color::WHITE }
        } else if is_empty_key {
            // Fade empty/transparent keys
            Color::from_rgba(OD_FG.0, OD_FG.1, OD_FG.2, alpha * 0.3)
        } else {
            Color::from_rgba(OD_FG.0, OD_FG.1, OD_FG.2, alpha)
        };

        // Uniform font size for all keys
        let tap_font_size = 12.0;

        // Check if this is a held layer key - show layer name prominently
        let show_layer_name = pressed && matches!(&label.hold, Some(HoldType::Layer(_, _)));

        // Hold label in bottom-right corner (if present and not showing layer name prominently)
        if let Some(hold) = label.hold {
            let (hold_label, hold_color, layer_idx) = match &hold {
                HoldType::Modifier(m) => {
                    (m.clone(), Color::from_rgba(0.6, 0.6, 0.6, alpha * 0.9), None)
                }
                HoldType::Layer(idx, name) => {
                    let lc = LAYER_COLORS.get(*idx).unwrap_or(&(0.5, 0.5, 0.5));
                    (name.clone(), Color::from_rgba(lc.0, lc.1, lc.2, alpha), Some(*idx))
                }
            };

            if show_layer_name {
                // When held, show layer name centered in background color
                let layer_text = container(text(hold_label).size(14.0).color(text_color))
                    .width(KEY_SIZE)
                    .height(KEY_SIZE)
                    .center_x(KEY_SIZE)
                    .center_y(KEY_SIZE);

                return container(layer_text)
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
                    .into();
            }

            // Tap label centered
            let tap_text = container(text(tap.clone()).size(tap_font_size).color(text_color))
                .width(KEY_SIZE)
                .height(KEY_SIZE)
                .center_x(KEY_SIZE)
                .center_y(KEY_SIZE);

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
            // No hold label - just show tap centered
            let tap_text = container(text(tap.clone()).size(tap_font_size).color(text_color))
                .width(KEY_SIZE)
                .height(KEY_SIZE)
                .center_x(KEY_SIZE)
                .center_y(KEY_SIZE);

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

    fn render_thumb_key(&self, label: KeyLabel, pressed: bool, base_hold: Option<HoldType>) -> Element<'static, Message> {
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
        let is_empty_key = tap.is_empty() || tap == " " || tap == "▽" || tap == "·";

        let text_color = if pressed {
            Color { a: alpha, ..Color::WHITE }
        } else if is_empty_key {
            // Fade empty/transparent keys
            Color::from_rgba(OD_FG.0, OD_FG.1, OD_FG.2, alpha * 0.3)
        } else {
            Color::from_rgba(OD_FG.0, OD_FG.1, OD_FG.2, alpha)
        };

        // Uniform font size for thumb keys
        let tap_font_size = 12.0;

        // Check if this is a held layer key - use base layer hold info if available
        // This handles the case where we've switched layers but want to show what layer key is being held
        let effective_hold = base_hold.or(label.hold.clone());
        let show_layer_name = pressed && matches!(&effective_hold, Some(HoldType::Layer(_, _)));

        // If pressed and this is a layer key (from base layer), show layer name prominently
        if show_layer_name {
            if let Some(HoldType::Layer(_idx, name)) = &effective_hold {
                let layer_text = container(text(name.clone()).size(14.0).color(text_color))
                    .width(KEY_SIZE)
                    .height(KEY_SIZE)
                    .center_x(KEY_SIZE)
                    .center_y(KEY_SIZE);

                return container(layer_text)
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
                    .into();
            }
        }

        // Hold label in bottom-right corner (if present)
        if let Some(hold) = label.hold {
            let (hold_label, hold_color) = match &hold {
                HoldType::Modifier(m) => {
                    (m.clone(), Color::from_rgba(0.6, 0.6, 0.6, alpha * 0.9))
                }
                HoldType::Layer(idx, name) => {
                    let lc = LAYER_COLORS.get(*idx).unwrap_or(&(0.5, 0.5, 0.5));
                    (name.clone(), Color::from_rgba(lc.0, lc.1, lc.2, alpha))
                }
            };

            // Tap label centered
            let tap_text = container(text(tap.clone()).size(tap_font_size).color(text_color))
                .width(KEY_SIZE)
                .height(KEY_SIZE)
                .center_x(KEY_SIZE)
                .center_y(KEY_SIZE);

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
            // No hold label - just show tap centered
            let tap_text = container(text(tap.clone()).size(tap_font_size).color(text_color))
                .width(KEY_SIZE)
                .height(KEY_SIZE)
                .center_x(KEY_SIZE)
                .center_y(KEY_SIZE);

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

        // Active layer gets colored background, inactive layers show colored border
        let (bg_color, border_color, text_color) = if is_active {
            // Active layer: colored background
            let bg = if layer_idx <= 2 {
                // BASE, EXTRA, TAP use lighter gray since their color is white
                Color::from_rgb(0.4, 0.4, 0.4)
            } else {
                Color::from_rgb(layer_color.0, layer_color.1, layer_color.2)
            };
            let border = Color::from_rgb(
                (layer_color.0 + 0.2).min(1.0),
                (layer_color.1 + 0.2).min(1.0),
                (layer_color.2 + 0.2).min(1.0),
            );
            // Dark text for bright backgrounds
            let luminance = 0.299 * layer_color.0 + 0.587 * layer_color.1 + 0.114 * layer_color.2;
            let txt = if luminance > 0.5 || layer_idx <= 2 {
                Color::from_rgb(0.1, 0.1, 0.1)
            } else {
                Color::WHITE
            };
            (bg, border, txt)
        } else {
            // Inactive layers: dark bg with colored border matching layer color
            let border = if layer_idx <= 2 {
                // BASE, EXTRA, TAP use dim gray border
                Color::from_rgb(OD_BORDER.0 * 0.5, OD_BORDER.1 * 0.5, OD_BORDER.2 * 0.5)
            } else {
                // Other layers show their color as a dim border
                Color::from_rgb(layer_color.0 * 0.5, layer_color.1 * 0.5, layer_color.2 * 0.5)
            };
            let txt = if layer_idx <= 2 {
                Color::from_rgb(0.4, 0.4, 0.4)
            } else {
                // Show layer color as dim text
                Color::from_rgb(layer_color.0 * 0.6, layer_color.1 * 0.6, layer_color.2 * 0.6)
            };
            (
                Color::from_rgb(OD_BG.0, OD_BG.1, OD_BG.2),  // dark bg
                border,
                txt,
            )
        };

        let label = label.to_string();

        // Fixed width for uniform sizing - wider to fit full names
        container(text(label).size(9).color(text_color))
            .width(62)
            .height(LAYER_INDICATOR_HEIGHT)
            .center_x(62)
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

        // Show title toggle
        let show_title_checkbox = checkbox("Show title", self.settings.show_title)
            .on_toggle(Message::ToggleShowTitle)
            .text_size(14)
            .style(move |_theme, status| {
                let icon_color = match status {
                    checkbox::Status::Active { is_checked: true } | checkbox::Status::Hovered { is_checked: true } => {
                        Color::from_rgb(0.596, 0.765, 0.475) // green
                    }
                    _ => text_color,
                };
                checkbox::Style {
                    background: Background::Color(bg_color),
                    icon_color,
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    text_color: Some(text_color),
                }
            });

        // Show layer indicators toggle
        let show_layers_checkbox = checkbox("Show layer indicators", self.settings.show_layer_indicators)
            .on_toggle(Message::ToggleShowLayerIndicators)
            .text_size(14)
            .style(move |_theme, status| {
                let icon_color = match status {
                    checkbox::Status::Active { is_checked: true } | checkbox::Status::Hovered { is_checked: true } => {
                        Color::from_rgb(0.596, 0.765, 0.475) // green
                    }
                    _ => text_color,
                };
                checkbox::Style {
                    background: Background::Color(bg_color),
                    icon_color,
                    border: Border {
                        color: border_color,
                        width: 1.0,
                        radius: 2.0.into(),
                    },
                    text_color: Some(text_color),
                }
            });

        // HID status with OneDark colors - check actual connection state
        let hid_status = if let Some(ref monitor) = self.hid_monitor {
            if monitor.is_connected() {
                text("HID: Connected").size(12).color(green_color)
            } else {
                text("HID: Reconnecting...").size(12).color(Color::from_rgb(0.898, 0.753, 0.482)) // yellow
            }
        } else {
            text("HID: Disabled").size(12).color(red_color)
        };

        let settings_content = column![
            header,
            container(
                column![
                    bg_trans_row,
                    Space::with_height(15),
                    key_trans_row,
                    Space::with_height(15),
                    show_title_checkbox,
                    Space::with_height(10),
                    show_layers_checkbox,
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
            // Keep keyboard subscription only for Escape key handling in settings
            keyboard::on_key_press(|key, _mods| Some(Message::KeyPressed(key))),
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

        // Add HID polling if connected - this handles layer changes and keypress highlighting
        // Using 4ms (250 Hz) for better responsiveness to quick keypresses
        if self.hid_monitor.is_some() {
            subs.push(time::every(Duration::from_millis(4)).map(|_| Message::HidTick));
        }

        Subscription::batch(subs)
    }
}
