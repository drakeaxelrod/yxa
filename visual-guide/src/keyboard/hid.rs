//! HID device communication for Yxa keyboard
//!
//! Monitors the keyboard's Raw HID interface to receive layer state and keypress events.

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

/// Message types from firmware (matching yxa_features.c)
const MSG_REQUEST_STATE: u8 = 0x00;
const MSG_LAYER_STATE: u8 = 0x01;
const MSG_KEY_PRESS: u8 = 0x02;
const MSG_KEY_RELEASE: u8 = 0x03;
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
                // Wait ~1 second before next attempt (60 polls at 16ms each)
                self.reconnect_cooldown = 60;
            } else {
                self.reconnect_cooldown -= 1;
            }
            return None;
        }

        let file = self.file.as_mut()?;
        let mut buffer = [0u8; 64];

        match file.read(&mut buffer) {
            Ok(n) if n >= 2 => {
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
                        self.pressed_keys.push((event.row, event.col));
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
                    _ => {}
                }
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
}

impl Drop for SyncHidMonitor {
    fn drop(&mut self) {
        self.file.take();
    }
}
