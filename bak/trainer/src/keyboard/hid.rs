use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

const MSG_LAYER_STATE: u8 = 0x01;

/// Find the SZR35 Raw HID interface
pub fn find_szr35_hidraw() -> Result<(PathBuf, usize)> {
    for i in 0..20 {
        let path = PathBuf::from(format!("/dev/hidraw{}", i));
        let uevent_path = format!("/sys/class/hidraw/hidraw{}/device/uevent", i);

        if !path.exists() {
            continue;
        }

        if let Ok(content) = fs::read_to_string(&uevent_path) {
            if content.contains("SZR35") && (content.contains("input1") || content.contains("input2"))
            {
                return Ok((path, i));
            }
        }
    }

    anyhow::bail!("SZR35 keyboard not found")
}

/// List available hidraw devices for debugging
pub fn list_hidraw_devices() -> Vec<(usize, String)> {
    let mut devices = Vec::new();

    for i in 0..20 {
        let uevent_path = format!("/sys/class/hidraw/hidraw{}/device/uevent", i);

        if let Ok(content) = fs::read_to_string(&uevent_path) {
            for line in content.lines() {
                if line.starts_with("HID_NAME=") {
                    let name = line.strip_prefix("HID_NAME=").unwrap_or(line);
                    devices.push((i, name.to_string()));
                    break;
                }
            }
        }
    }

    devices
}

/// HID monitor that reads layer state from the keyboard
pub struct HidMonitor {
    running: Arc<AtomicBool>,
    sender: mpsc::Sender<u8>,
}

impl HidMonitor {
    pub fn new(sender: mpsc::Sender<u8>) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            sender,
        }
    }

    /// Start monitoring in a background thread
    pub fn start(&self) -> Result<()> {
        let (path, _num) = find_szr35_hidraw().context("Failed to find SZR35 keyboard")?;

        self.running.store(true, Ordering::SeqCst);
        let running = Arc::clone(&self.running);
        let sender = self.sender.clone();

        std::thread::spawn(move || {
            if let Err(e) = monitor_loop(&path, running, sender) {
                eprintln!("HID monitor error: {}", e);
            }
        });

        Ok(())
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

fn monitor_loop(
    path: &PathBuf,
    running: Arc<AtomicBool>,
    sender: mpsc::Sender<u8>,
) -> Result<()> {
    // Open with O_NONBLOCK
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)
        .context("Failed to open HID device (permission denied?)")?;

    // Send initial request for layer state
    let request = [0u8; 32];
    let _ = std::io::Write::write(&mut file, &request);

    let mut buffer = [0u8; 64];
    let mut current_layer = 0u8;

    while running.load(Ordering::SeqCst) {
        // Non-blocking read with polling
        match file.read(&mut buffer) {
            Ok(n) if n >= 2 => {
                // MSG_LAYER_STATE = 0x01, byte 1 = layer number
                if buffer[0] == MSG_LAYER_STATE && buffer[1] <= 7 {
                    let new_layer = buffer[1];
                    if new_layer != current_layer {
                        current_layer = new_layer;
                        let _ = sender.blocking_send(new_layer);
                    }
                }
            }
            Ok(_) => {}
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No data available, sleep briefly
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }

    Ok(())
}

/// Synchronous HID monitor for the terminal trainer
pub struct SyncHidMonitor {
    file: Option<File>,
    current_layer: u8,
}

impl SyncHidMonitor {
    pub fn new() -> Result<Self> {
        let (path, _) = find_szr35_hidraw()?;

        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&path)
            .context("Failed to open HID device")?;

        // Send initial request
        let request = [0u8; 32];
        let _ = (&file).write_all(&request);

        Ok(Self {
            file: Some(file),
            current_layer: 0,
        })
    }

    /// Poll for layer changes, returns Some(layer) if changed
    pub fn poll(&mut self) -> Option<u8> {
        let file = self.file.as_mut()?;
        let mut buffer = [0u8; 64];

        match file.read(&mut buffer) {
            Ok(n) if n >= 2 => {
                if buffer[0] == MSG_LAYER_STATE && buffer[1] <= 7 {
                    let new_layer = buffer[1];
                    if new_layer != self.current_layer {
                        self.current_layer = new_layer;
                        return Some(new_layer);
                    }
                }
            }
            _ => {}
        }

        None
    }

    pub fn current_layer(&self) -> u8 {
        self.current_layer
    }
}

use std::io::Write;

impl Drop for SyncHidMonitor {
    fn drop(&mut self) {
        self.file.take();
    }
}
