# SZR35 Miryoku

SZR35 split keyboard with Miryoku layout, layer overlay, and RGB layer indication.

## Quick Start

```bash
# Enter the development shell
nix develop

# Run the terminal trainer with auto layer detection
trainer-hid

# Run the GUI overlay with auto layer detection
overlay

# Run the terminal trainer in manual mode (press 0-7 to view layers)
trainer

# Flash firmware (keyboard must be in DFU mode)
flash
```

## Miryoku Layout

This uses the standard [Miryoku](https://github.com/manna-harbour/miryoku) layout for split_3x5_3:

| Layer | Thumb Key | Active Hand | Color |
|-------|-----------|-------------|-------|
| 0 - BASE | - | Both | Finger colors |
| 1 - NAV | Space | Right | Cyan |
| 2 - MOUSE | Tab | Right | Green |
| 3 - MEDIA | Escape | Right | Magenta |
| 4 - NUM | Backspace | Left | Yellow |
| 5 - SYM | Enter | Left | Red |
| 6 - FUN | Delete | Left | Blue |
| 7 - BUTTON | Z / / | Both | Orange |

## Project Structure

```
szr35-miryoku/
├── firmware/
│   ├── keymap.c               # QMK keymap with layer broadcast + RGB
│   └── szrkbd_szr35_vial.bin  # Compiled firmware
├── layouts/
│   └── miryoku-kbd-layout.vil # Miryoku layout for Vial (split_3x5_3)
├── overlay/
│   ├── miryoku_overlay.py     # GUI layer overlay (PyQt6)
│   ├── miryoku_trainer.py     # Terminal layer trainer (Rich)
│   └── hid_test.py            # HID debug tool
├── flake.nix                  # Nix flake for dependencies
└── README.md
```

## Features

- **Layer Broadcast**: Firmware sends current layer over Raw HID to overlay/trainer
- **RGB Layer Indication**: LEDs change color based on active layer
- **Hot Reload**: Trainer/overlay reload layout file when modified
- **Direct HID Access**: Works on NixOS without hidapi (uses /dev/hidraw directly)

## HID Permissions

If the overlay/trainer can't access the keyboard:

```bash
sudo chmod 666 /dev/hidraw*
```

Or add a udev rule for permanent access:

```bash
# /etc/udev/rules.d/99-szr35.rules
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="3601", ATTRS{idProduct}=="45d4", MODE="0666"
```

## Entering DFU Mode

To flash firmware, you need to enter DFU mode using the boot pads:

1. **Locate the boot pads**: Look for the white square with two dots. It's near the thumb cluster, slightly to the side (NOT the one opposite the USB port).
2. **Short the pads**: Use tweezers, a paperclip, or tin foil to bridge the two pads.
3. **While keeping them shorted**, plug in the USB cable.
4. **Release** after the keyboard is plugged in.
5. Run `flash` command.

Each half must be flashed separately.

## Loading Miryoku Layout

The Miryoku layout is stored in the VIL file and loaded via Vial:

1. Open Vial application
2. Load `layouts/miryoku-kbd-layout.vil`
3. Layout is saved to keyboard EEPROM

The keymap.c provides the base firmware with layer broadcast and RGB support. The actual key mappings come from the VIL file loaded via Vial.

## Building Firmware

To rebuild firmware with changes:

```bash
build  # Uses Docker to compile QMK firmware
```

This requires:
- Docker installed
- vial-qmk source with SZR35 keyboard definition

The build command runs:
```bash
docker run --rm -v /path/to/vial-qmk:/qmk_firmware qmkfm/qmk_cli:latest make szrkbd/szr35:vial
```

### Firmware Source

The keymap.c in `firmware/` includes:
- Layer broadcast over Raw HID (for overlay communication)
- RGB Matrix layer indication (different colors per layer)
- Base keymap (actual layout comes from Vial/VIL file)
