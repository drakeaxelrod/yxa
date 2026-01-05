//! HID device communication for Yxa keyboard
//!
//! Monitors the keyboard's Raw HID interface to receive layer state and keypress events.

use anyhow::Result;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

/// Message types from firmware (matching yxa_features.c)
const MSG_REQUEST_STATE: u8 = 0x00;
const MSG_LAYER_STATE: u8 = 0x01;
const MSG_KEY_PRESS: u8 = 0x02;
const MSG_KEY_RELEASE: u8 = 0x03;
const MSG_CAPS_WORD_STATE: u8 = 0x04;
const MSG_MODIFIER_STATE: u8 = 0x05;
const MSG_HEARTBEAT: u8 = 0x06;
const MSG_FULL_STATE: u8 = 0x07;
const MSG_KEY_BATCH: u8 = 0x08;
const MSG_TOGGLE_KEYPRESS: u8 = 0x10;

/// A key event from the keyboard
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyEvent {
    pub row: u8,
    pub col: u8,
    pub keycode: u16,
    pub pressed: bool,
}

/// Events received from the keyboard
#[derive(Debug, Clone)]
pub enum HidEvent {
    LayerChange(u8),
    KeyPress(KeyEvent),
    KeyRelease(KeyEvent),
    CapsWordState(bool),
    ModifierState(u8),
    FullState {
        layer: u8,
        caps_word: bool,
        modifiers: u8,
        pressed_keys: Vec<(u8, u8)>,
    },
}

/// Known keyboard product names to search for
const KEYBOARD_NAMES: &[&str] = &["Yxa", "SZR35"];

/// Find the Yxa keyboard Raw HID interface
///
/// Searches /dev/hidraw* devices for a keyboard matching known names.
/// Returns the device path and hidraw number on success.
pub fn find_keyboard_hidraw() -> Result<(PathBuf, usize)> {
    for i in 0..20 {
        let path = PathBuf::from(format!("/dev/hidraw{}", i));
        let uevent_path = format!("/sys/class/hidraw/hidraw{}/device/uevent", i);

        if !path.exists() {
            continue;
        }

        if let Ok(content) = std::fs::read_to_string(&uevent_path) {
            // Check if this is one of our keyboards
            let is_our_keyboard = KEYBOARD_NAMES.iter().any(|name| content.contains(name));
            let is_raw_hid = content.contains("input1") || content.contains("input2");

            if is_our_keyboard && is_raw_hid {
                return Ok((path, i));
            }
        }
    }

    anyhow::bail!(
        "Keyboard not found. Looking for devices named: {}",
        KEYBOARD_NAMES.join(", ")
    )
}

/// Synchronous HID monitor for reading keyboard events
///
/// Uses non-blocking I/O to poll for layer state and keypress events from the keyboard firmware.
/// Automatically attempts to reconnect if the keyboard is disconnected.
pub struct SyncHidMonitor {
    file: Option<File>,
    current_layer: u8,
    /// Currently pressed keys (row, col) for highlighting
    pressed_keys: Vec<(u8, u8)>,
    /// Tracks if we're currently connected
    connected: bool,
    /// Counter for reconnection attempts (to avoid spamming)
    reconnect_cooldown: u8,
    /// Caps Word state
    caps_word_active: bool,
    /// Current modifier state (bitmask)
    modifier_state: u8,
    /// Last received sequence number (for detecting dropped packets)
    last_sequence: u8,
    /// Count of detected dropped packets
    dropped_packets: u32,
}

impl SyncHidMonitor {
    /// Create a new HID monitor connected to the keyboard
    pub fn new() -> Result<Self> {
        let mut monitor = Self {
            file: None,
            current_layer: 0,
            pressed_keys: Vec::new(),
            connected: false,
            reconnect_cooldown: 0,
            caps_word_active: false,
            modifier_state: 0,
            last_sequence: 0,
            dropped_packets: 0,
        };
        monitor.try_connect();
        Ok(monitor)
    }

    /// Attempt to connect to the keyboard
    fn try_connect(&mut self) -> bool {
        if let Ok((path, _)) = find_keyboard_hidraw() {
            if let Ok(file) = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&path)
            {
                // Send initial request for layer state
                let mut request = [0u8; 32];
                request[0] = MSG_REQUEST_STATE;
                let _ = (&file).write_all(&request);

                self.file = Some(file);
                self.connected = true;
                self.pressed_keys.clear();
                return true;
            }
        }
        self.connected = false;
        false
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Toggle keypress broadcasting on the keyboard
    pub fn toggle_keypress_broadcast(&mut self) -> Result<()> {
        if let Some(file) = self.file.as_mut() {
            let mut request = [0u8; 32];
            request[0] = MSG_TOGGLE_KEYPRESS;
            file.write_all(&request)?;
        }
        Ok(())
    }

    /// Parse a single HID message from buffer and return the event
    fn parse_message(&mut self, buffer: &[u8], n: usize) -> Option<HidEvent> {
        if n < 2 {
            return None;
        }

        match buffer[0] {
            MSG_LAYER_STATE if buffer[1] <= 9 => {
                let new_layer = buffer[1];
                if new_layer != self.current_layer {
                    self.current_layer = new_layer;
                    return Some(HidEvent::LayerChange(new_layer));
                }
            }
            MSG_KEY_PRESS if n >= 3 => {
                let event = KeyEvent {
                    row: buffer[1],
                    col: buffer[2],
                    keycode: 0,
                    pressed: true,
                };
                // Avoid duplicates
                if !self.pressed_keys.contains(&(event.row, event.col)) {
                    self.pressed_keys.push((event.row, event.col));
                }
                return Some(HidEvent::KeyPress(event));
            }
            MSG_KEY_RELEASE if n >= 3 => {
                let event = KeyEvent {
                    row: buffer[1],
                    col: buffer[2],
                    keycode: 0,
                    pressed: false,
                };
                self.pressed_keys.retain(|&(r, c)| r != event.row || c != event.col);
                return Some(HidEvent::KeyRelease(event));
            }
            MSG_CAPS_WORD_STATE if n >= 2 => {
                let active = buffer[1] != 0;
                if active != self.caps_word_active {
                    self.caps_word_active = active;
                    return Some(HidEvent::CapsWordState(active));
                }
            }
            MSG_MODIFIER_STATE if n >= 2 => {
                let mods = buffer[1];
                if mods != self.modifier_state {
                    self.modifier_state = mods;
                    return Some(HidEvent::ModifierState(mods));
                }
            }
            MSG_FULL_STATE if n >= 4 => {
                // Full state response: layer, caps_word, modifiers, key_count, [keys...]
                let layer = buffer[1];
                let caps_word = buffer[2] != 0;
                let modifiers = buffer[3];
                let mut pressed_keys = Vec::new();

                if n >= 5 {
                    let key_count = buffer[4] as usize;
                    for i in 0..key_count {
                        let idx = 5 + i * 2;
                        if idx + 1 < n {
                            pressed_keys.push((buffer[idx], buffer[idx + 1]));
                        }
                    }
                }

                self.current_layer = layer;
                self.caps_word_active = caps_word;
                self.modifier_state = modifiers;
                self.pressed_keys = pressed_keys.clone();

                return Some(HidEvent::FullState {
                    layer,
                    caps_word,
                    modifiers,
                    pressed_keys,
                });
            }
            MSG_KEY_BATCH if n >= 2 => {
                // Batch format: count, [type, row, col] * count
                let count = buffer[1] as usize;
                let mut events = Vec::new();

                for i in 0..count {
                    let idx = 2 + i * 3;
                    if idx + 2 < n {
                        let event_type = buffer[idx];
                        let row = buffer[idx + 1];
                        let col = buffer[idx + 2];

                        let event = KeyEvent {
                            row,
                            col,
                            keycode: 0,
                            pressed: event_type == MSG_KEY_PRESS,
                        };

                        if event_type == MSG_KEY_PRESS {
                            if !self.pressed_keys.contains(&(row, col)) {
                                self.pressed_keys.push((row, col));
                            }
                            events.push(HidEvent::KeyPress(event));
                        } else if event_type == MSG_KEY_RELEASE {
                            self.pressed_keys.retain(|&(r, c)| r != row || c != col);
                            events.push(HidEvent::KeyRelease(event));
                        }
                    }
                }

                // Return the first event; the rest will be handled by poll_all_events
                // In practice, batch messages are fully processed there
                return events.into_iter().next();
            }
            _ => {}
        }

        None
    }

    /// Poll for any keyboard events
    ///
    /// Returns `Some(HidEvent)` if an event occurred, `None` otherwise.
    /// Automatically attempts to reconnect if disconnected.
    pub fn poll_event(&mut self) -> Option<HidEvent> {
        // If not connected, try to reconnect periodically
        if !self.connected || self.file.is_none() {
            if self.reconnect_cooldown == 0 {
                if self.try_connect() {
                    // Successfully reconnected
                    return None;
                }
                // Wait ~250ms before next attempt (at 4ms polling = 62 polls)
                self.reconnect_cooldown = 62;
            } else {
                self.reconnect_cooldown -= 1;
            }
            return None;
        }

        let file = self.file.as_mut()?;
        let mut buffer = [0u8; 64];

        match file.read(&mut buffer) {
            Ok(n) if n >= 2 => {
                return self.parse_message(&buffer, n);
            }
            Ok(0) => {
                // EOF - device disconnected
                self.file = None;
                self.connected = false;
                self.pressed_keys.clear();
            }
            Err(e) => {
                // Check for device disconnection errors
                if e.kind() == std::io::ErrorKind::BrokenPipe
                    || e.kind() == std::io::ErrorKind::NotConnected
                    || e.raw_os_error() == Some(libc::ENODEV)
                    || e.raw_os_error() == Some(libc::ENXIO)
                {
                    self.file = None;
                    self.connected = false;
                    self.pressed_keys.clear();
                }
                // WouldBlock is normal for non-blocking I/O
            }
            _ => {}
        }

        None
    }

    /// Poll for ALL buffered keyboard events at once
    ///
    /// This drains the kernel HID buffer completely, preventing event loss
    /// during rapid typing. Returns a vector of all pending events.
    pub fn poll_all_events(&mut self) -> Vec<HidEvent> {
        let mut events = Vec::new();

        // If not connected, try to reconnect periodically
        if !self.connected || self.file.is_none() {
            if self.reconnect_cooldown == 0 {
                if self.try_connect() {
                    // Successfully reconnected, request full state
                    self.request_full_state();
                    return events;
                }
                // Wait ~250ms before next attempt (at 4ms polling = 62 polls)
                self.reconnect_cooldown = 62;
            } else {
                self.reconnect_cooldown -= 1;
            }
            return events;
        }

        // Collect all buffers first, then process them
        let mut buffers: Vec<([u8; 64], usize)> = Vec::new();

        if let Some(file) = self.file.as_mut() {
            loop {
                let mut buffer = [0u8; 64];
                match file.read(&mut buffer) {
                    Ok(n) if n >= 2 => {
                        buffers.push((buffer, n));
                    }
                    Ok(0) => {
                        // EOF - device disconnected
                        self.file = None;
                        self.connected = false;
                        self.pressed_keys.clear();
                        break;
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock {
                            // No more data available - this is normal
                            break;
                        }
                        // Check for device disconnection errors
                        if e.kind() == std::io::ErrorKind::BrokenPipe
                            || e.kind() == std::io::ErrorKind::NotConnected
                            || e.raw_os_error() == Some(libc::ENODEV)
                            || e.raw_os_error() == Some(libc::ENXIO)
                        {
                            self.file = None;
                            self.connected = false;
                            self.pressed_keys.clear();
                        }
                        break;
                    }
                    _ => break,
                }
            }
        }

        // Now process all collected buffers
        for (buffer, n) in buffers {
            // Handle batch messages specially - they contain multiple events
            if buffer[0] == MSG_KEY_BATCH && n >= 2 {
                let count = buffer[1] as usize;
                for i in 0..count {
                    let idx = 2 + i * 3;
                    if idx + 2 < n {
                        let event_type = buffer[idx];
                        let row = buffer[idx + 1];
                        let col = buffer[idx + 2];

                        let event = KeyEvent {
                            row,
                            col,
                            keycode: 0,
                            pressed: event_type == MSG_KEY_PRESS,
                        };

                        if event_type == MSG_KEY_PRESS {
                            if !self.pressed_keys.contains(&(row, col)) {
                                self.pressed_keys.push((row, col));
                            }
                            events.push(HidEvent::KeyPress(event));
                        } else if event_type == MSG_KEY_RELEASE {
                            self.pressed_keys.retain(|&(r, c)| r != row || c != col);
                            events.push(HidEvent::KeyRelease(event));
                        }
                    }
                }
            } else if let Some(event) = self.parse_message(&buffer, n) {
                events.push(event);
            }
        }

        events
    }

    /// Request full state from keyboard (layer, caps word, modifiers, pressed keys)
    pub fn request_full_state(&mut self) {
        if let Some(file) = self.file.as_mut() {
            let mut request = [0u8; 32];
            request[0] = MSG_REQUEST_STATE;
            let _ = file.write_all(&request);
        }
    }

    /// Send a heartbeat ping to the keyboard
    pub fn send_heartbeat(&mut self) {
        if let Some(file) = self.file.as_mut() {
            let mut request = [0u8; 32];
            request[0] = MSG_HEARTBEAT;
            let _ = file.write_all(&request);
        }
    }

    /// Poll for layer changes only (backwards compatible)
    ///
    /// Returns `Some(layer)` if the layer changed since last poll, `None` otherwise.
    pub fn poll(&mut self) -> Option<u8> {
        match self.poll_event() {
            Some(HidEvent::LayerChange(layer)) => Some(layer),
            _ => None,
        }
    }

    /// Get the current layer number (0-9)
    pub fn current_layer(&self) -> u8 {
        self.current_layer
    }

    /// Get currently pressed keys as (row, col) pairs
    pub fn pressed_keys(&self) -> &[(u8, u8)] {
        &self.pressed_keys
    }

    /// Check if a specific key is currently pressed
    pub fn is_key_pressed(&self, row: u8, col: u8) -> bool {
        self.pressed_keys.iter().any(|&(r, c)| r == row && c == col)
    }

    /// Check if Caps Word is currently active
    pub fn is_caps_word_active(&self) -> bool {
        self.caps_word_active
    }

    /// Get current modifier state (bitmask)
    pub fn modifier_state(&self) -> u8 {
        self.modifier_state
    }

    /// Get count of dropped packets detected
    pub fn dropped_packets(&self) -> u32 {
        self.dropped_packets
    }
}

impl Drop for SyncHidMonitor {
    fn drop(&mut self) {
        self.file.take();
    }
}
