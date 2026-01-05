# Yxa

**Yxa** (Swedish for "Axe") — A 36-Key Split Ergonomic Keyboard

A complete software suite for the Yxa split keyboard, featuring Keyberon-based firmware and a visual layout guide.

## Crates

| Crate | Description |
|-------|-------------|
| `yxa-firmware` | Keyberon firmware for STM32F401 |
| `yxa-visual-guide` | Layout trainer/overlay GUI |

## About the Hardware

The Yxa keyboard is based on the **SZR35** (also sold as "Hardtochooseone") split ergonomic keyboard. The original hardware was sourced from AliExpress:

- **Original Product**: [AliExpress Listing](https://www.aliexpress.com/item/1005008486363216.html)
- **Seller**: Store selling under "Hardtochooseone" / "SZR35" branding
- **Original Name**: SZR35 (despite having 36 keys, not 35)

This project renames the keyboard to **Yxa** for personal use and to better reflect the 36-key count. All credit for the hardware design goes to the original creators.

## Hardware Specifications

| Feature | Specification |
|---------|---------------|
| Layout | 36-Key Split Ergonomic |
| MCU | STM32F401CCU6 |
| Connection | Wired USB-C |
| Inter-half | TRS Cable |
| Switches | Hot-Swappable MX Compatible |
| Backlighting | Per-Key North RGB |
| Case | 3D Printed |
| Keycaps | XDA Profile (Included) |

## Bootloader

The keyboard comes with **PlumBL** bootloader pre-installed. However, the STM32F401 has a **built-in DFU bootloader** in ROM, making PlumBL unnecessary. This firmware uses the native STM32 DFU bootloader instead:

- **DFU Mode**: Hold BOOT0 button while connecting USB, or trigger via software reset
- **No external bootloader needed**: The STM32F401's ROM bootloader is always available
- **Flashing**: Use `dfu-util` or STM32CubeProgrammer

## Development

Using Nix flakes:

```bash
# Enter development shell
nix develop

# Build firmware
build-firmware

# Run visual guide
guide
```

Without Nix:

```bash
# Install embedded target
rustup target add thumbv7em-none-eabihf

# Build firmware
cargo build --release -p yxa-firmware

# Convert to binary
arm-none-eabi-objcopy -O binary \
  target/thumbv7em-none-eabihf/release/yxa-firmware \
  yxa-firmware.bin

# Run visual guide
cargo run --release -p yxa-visual-guide
```

## Flashing

1. Enter DFU mode (short BOOT0 pads while connecting USB)
2. Flash with dfu-util:
```bash
dfu-util -a 0 -s 0x08000000:leave -D yxa-firmware.bin
```

**Note**: Each half must be flashed separately.

## Features

- **Hot-Swappable**: MX hot-swap sockets for easy switch replacement
- **RGB Backlighting**: North-facing per-key RGB LEDs
- **Split Design**: Ergonomic split layout reduces wrist strain
- **USB-C**: Modern USB-C connectivity (either half can be master)
- **Keyberon Firmware**: Pure Rust firmware with full customization
- **Visual Guide**: Interactive overlay for learning your layout

## Attribution

This project is a custom firmware implementation for hardware designed and sold by third parties. The Yxa name is used for this personal project only. The original hardware:

- Is sold under the names "SZR35" and "Hardtochooseone"
- Was designed by the original creators (unknown attribution)
- Is available from various AliExpress sellers

No trademark or copyright infringement is intended. This software is provided as-is for personal use.

## License

This software is dual-licensed under MIT or Apache-2.0, at your option.

The hardware design remains the property of its original creators.
