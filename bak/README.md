# SZR35 Miryoku

Complete setup for the SZR35 split keyboard with Miryoku layout, RGB layer indication, layer broadcasting, and Vial support.

![Miryoku Layout](miryoku-kle-cover-miryoku_qmk.png)

## Quick Start

```bash
# Enter the development shell
nix develop

# Train with the layout (auto-detects layer from keyboard)
trainer-hid

# Build firmware (first run clones vial-qmk, ~500MB)
build

# Flash firmware (keyboard must be in DFU mode)
flash
```

## Features

- **Miryoku Layout**: Full [Miryoku](https://github.com/manna-harbour/miryoku) `split_3x5_3` with Colemak-DH
- **Home Row Mods**: GUI/Alt/Ctrl/Shift on home row
- **RGB Layer Colors**: Visual feedback showing active layer
- **Layer Broadcasting**: Firmware sends layer changes over Raw HID
- **Vial Support**: Edit keymap in real-time with Vial app
- **Training Tools**: Terminal trainer and GUI overlay

## Miryoku Layers

| Layer | Index | Thumb Key | Hand | RGB Color |
|-------|-------|-----------|------|-----------|
| BASE | 0 | - | Both | Per-finger |
| NAV | 1 | Space | Right | Cyan |
| MOUSE | 2 | Tab | Right | Green |
| MEDIA | 3 | Escape | Right | Magenta |
| NUM | 4 | Backspace | Left | Yellow |
| SYM | 5 | Enter | Left | Red |
| FUN | 6 | Delete | Left | Blue |
| BUTTON | 7 | Z or / | Both | Orange |

### Layer Access

```
Left Thumb:  Esc→MEDIA  Space→NAV  Tab→MOUSE
Right Thumb: Enter→SYM  Bksp→NUM   Del→FUN
Pinkies:     Z→BUTTON               /→BUTTON
```

## Project Structure

```
szr35-miryoku/
├── firmware/
│   └── szrkbd_szr35_vial.bin     # Ready-to-flash firmware
├── layouts/
│   └── miryoku-kbd-layout.vil    # Vial layout file
├── overlay/
│   ├── miryoku_overlay.py        # GUI overlay (PyQt6)
│   └── miryoku_trainer.py        # Terminal trainer (Rich)
├── qmk/
│   └── szrkbd/szr35/             # Complete keyboard definition
│       ├── keyboard.json         # Matrix, RGB, split config
│       ├── keymaps/vial/
│       │   ├── keymap.c          # Miryoku + broadcast + RGB
│       │   ├── rules.mk          # Features enabled
│       │   └── vial.json         # Vial definition
│       └── ld/                   # Linker scripts
├── flake.nix                     # Nix dev environment
└── README.md
```

## Commands

All commands work inside `nix develop`:

| Command | Description |
|---------|-------------|
| `trainer-hid` | Terminal trainer with auto layer detection |
| `trainer` | Terminal trainer in manual mode (press 0-7) |
| `overlay` | GUI layer overlay |
| `build` | Build firmware using Docker |
| `flash` | Flash firmware to keyboard |
| `fix-hid` | Fix HID permissions for overlay |

## Building Firmware

```bash
nix develop
build
```

First run clones vial-qmk to `~/.cache/szr35-vial-qmk` (~500MB, one-time).

Requirements:
- Docker installed and running
- Nix with flakes enabled

## Flashing Firmware

### Enter DFU Mode

1. **Locate boot pads**: White square with 2 dots near thumb cluster
   - It's the one **near the thumb cluster**, NOT opposite USB port
2. **Short the pads** with tweezers or paperclip
3. **While shorted**, plug in USB cable
4. **Release** after keyboard is connected
5. Run `flash`

**Each half must be flashed separately.**

### If Keyboard Won't Enter DFU

- Use a data USB cable (not charge-only)
- Hold boot pads for 5+ seconds while connecting
- Verify with: `lsusb | grep -i stm`
- Try: `sudo dmesg | tail` to check USB connection

### If Keyboard is Bricked

1. Unplug keyboard
2. Short boot pads firmly
3. While shorted, connect USB
4. Wait 5 seconds, then release
5. Run: `flash`

The STM32F401 has a built-in DFU bootloader that cannot be erased. (STM32 BOOTLOADER)

## Vial Layout

The keymap is compiled into firmware, but you can also use Vial for real-time editing:

1. Download [Vial](https://get.vial.today/)
2. Connect keyboard
3. Load `layouts/miryoku-kbd-layout.vil` (optional, for backup)
4. Edit layout - changes save to keyboard EEPROM

## Trainer & Overlay

### Terminal Trainer

```bash
trainer-hid    # Auto-detects active layer from keyboard
trainer        # Manual mode (press 0-7 to view layers)
```

### GUI Overlay

```bash
overlay        # Floating window showing current layer
```

### HID Permissions

If trainer/overlay can't access keyboard:

```bash
fix-hid        # Quick fix (temporary)
```

## Firmware Features

### RGB Layer Indication

- **Base layer**: Per-finger colors
  - Pinky: Cyan | Ring: Magenta | Middle: Green | Index: Yellow | Thumb: Blue
- **Other layers**: Solid color (see table above)

### Home Row Mods

| Finger | Left Hand | Right Hand |
|--------|-----------|------------|
| Pinky | GUI (A) | GUI (O) |
| Ring | Alt (R) | Alt (I) |
| Middle | Ctrl (S) | Ctrl (E) |
| Index | Shift (T) | Shift (N) |

### Layer Broadcasting

Firmware broadcasts active layer over Raw HID:
- Byte 0: `0x01` = layer state message
- Byte 1: layer number (0-7)

This allows overlay/trainer to show current layer automatically.

### Other Features

- **Caps Word** (`CW_TOGG`): Auto-capitalizes until space/punctuation
- **Mouse Keys**: Full mouse control on MOUSE layer
- **Media Keys**: Volume, playback on MEDIA layer
- **VialRGB**: Control RGB through Vial app

## Hardware

- **MCU**: STM32F401 (256KB flash, 64KB RAM)
- **Bootloader**: Native STM32 DFU (0x08000000)
- **Split**: USART serial (pin A9)
- **RGB**: WS2812 (pin A7), 18 LEDs per half
- **Crystal**: 16MHz external

## Troubleshooting

### Keyboard Not Detected

1. Check USB cable (must be data cable)
2. Try different USB port
3. Run `lsusb` to see if device appears

### DFU Flash Fails

1. Make sure keyboard is in DFU mode (`lsusb | grep STM`)
2. Run flash with sudo: `flash`
3. Try: `sudo dfu-util -l` to list DFU devices

### Overlay Can't Connect

1. Run `fix-hid` to set permissions
2. Check if keyboard is connected: `ls /dev/hidraw*`
3. Verify SZR35 is detected: `cat /sys/class/hidraw/hidraw*/device/uevent | grep SZR`

### Build Fails

1. Make sure Docker is running: `docker ps`
2. Check disk space (vial-qmk needs ~500MB)
3. Clear cache and retry: `rm -rf ~/.cache/szr35-vial-qmk`

## Credits & Inspiration

This project builds upon the work of several amazing open-source projects:

- **[Miryoku](https://github.com/manna-harbour/miryoku)** by Manna Harbour - The ergonomic keyboard layout system this firmware implements
- **[Miryoku QMK](https://github.com/manna-harbour/miryoku_qmk)** - Reference Miryoku implementation for QMK
- **[Vial](https://get.vial.today/)** - Real-time keyboard configuration
- **[Vial QMK](https://github.com/vial-kb/vial-qmk)** - QMK fork with Vial support
- **[QMK Firmware](https://github.com/qmk/qmk_firmware)** - The keyboard firmware framework

Layout visualization from [Miryoku KLE](https://github.com/manna-harbour/miryoku/tree/master/docs/reference).

## License

GPL-2.0-or-later (QMK compatible)
