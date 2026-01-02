#!/usr/bin/env python3
"""
Miryoku Layout Trainer
Displays the current layer with color-coded columns when you switch layers.

Usage:
  python miryoku_trainer.py              # Interactive mode (press 0-7 for layers)
  python miryoku_trainer.py --hid        # HID mode (auto-detect layer from keyboard)
"""

import argparse
import json
import subprocess
import re
import os
import sys
import select
import termios
import tty
import threading
from pathlib import Path

from rich.console import Console
from rich.table import Table

# Force unbuffered output
sys.stdout.reconfigure(line_buffering=True)

# Find the .vil file relative to the script
SCRIPT_DIR = Path(__file__).parent
VIL_FILE = SCRIPT_DIR.parent / 'layouts' / 'miryoku-kbd-layout.vil'

# Column colors for visual learning (finger assignment)
COL_COLORS = {
    0: "bright_cyan",      # Left pinky
    1: "bright_magenta",   # Left ring
    2: "bright_green",     # Left middle
    3: "bright_yellow",    # Left index
    4: "bright_yellow",    # Left index (inner)
    5: "bright_yellow",    # Right index (inner)
    6: "bright_yellow",    # Right index
    7: "bright_green",     # Right middle
    8: "bright_magenta",   # Right ring
    9: "bright_cyan",      # Right pinky
}

THUMB_COLOR = "bright_blue"

LAYER_COLORS = {
    0: "white",
    1: "bright_cyan",
    2: "bright_green",
    3: "bright_magenta",
    4: "bright_yellow",
    5: "bright_red",
    6: "bright_blue",
    7: "orange1",
    8: "white",
}

LAYER_NAMES = [
    "BASE (Colemak-DH)",
    "NAV ← ↓ ↑ →",
    "MOUSE",
    "MEDIA ♫",
    "NUM 123",
    "SYM !@#",
    "FUN F1-F12",
    "BUTTON",
    "EXTRA"
]

LAYER_ACTIVE_HAND = {
    0: "both",
    1: "right",
    2: "right",
    3: "right",
    4: "left",
    5: "left",
    6: "left",
    7: "both",
}


def simplify_keycode(kc):
    """Convert QMK keycode to readable label"""
    if kc == -1 or kc == "KC_NO":
        return "·"
    if kc == "KC_TRNS":
        return "▽"

    kc = str(kc)
    kc = re.sub(r'^KC_', '', kc)

    # Mod-taps: LGUI_T(KC_A) -> A/Gui
    if m := re.match(r'(\w+)_T\(KC_(\w+)\)', kc):
        mod, key = m.groups()
        mod_map = {'LGUI': 'Gui', 'LALT': 'Alt', 'LCTL': 'Ctl', 'LSFT': 'Sft', 'RALT': 'AGr'}
        return f"{key}/{mod_map.get(mod, mod)}"

    # Layer-taps: LT(1,KC_SPACE) -> Spc/L1
    if m := re.match(r'LT\((\d+),KC_(\w+)\)', kc):
        layer, key = m.groups()
        key_map = {'SPACE': 'Spc', 'ESCAPE': 'Esc', 'TAB': 'Tab', 'ENTER': 'Ent',
                   'BSPACE': 'Bsp', 'DELETE': 'Del', 'Z': 'Z', 'SLASH': '/'}
        return f"{key_map.get(key, key)}/L{layer}"

    simplify = {
        'SPACE': 'Spc', 'ESCAPE': 'Esc', 'BSPACE': 'Bsp', 'DELETE': 'Del',
        'ENTER': 'Ent', 'TAB': 'Tab', 'INSERT': 'Ins', 'HOME': 'Hom',
        'END': 'End', 'PGUP': 'PgU', 'PGDOWN': 'PgD',
        'LEFT': '←', 'RIGHT': '→', 'UP': '↑', 'DOWN': '↓',
        'LSHIFT': 'Sft', 'LCTRL': 'Ctl', 'LALT': 'Alt', 'LGUI': 'Gui', 'RALT': 'AGr',
        'QUOTE': "'", 'COMMA': ',', 'DOT': '.', 'SLASH': '/', 'SCOLON': ';',
        'LBRACKET': '[', 'RBRACKET': ']', 'BSLASH': '\\', 'GRAVE': '`',
        'EQUAL': '=', 'MINUS': '-',
        'LCBR': '{', 'RCBR': '}', 'LPRN': '(', 'RPRN': ')',
        'AMPR': '&', 'ASTR': '*', 'COLN': ':', 'DLR': '$',
        'PERC': '%', 'CIRC': '^', 'PLUS': '+', 'TILD': '~',
        'EXLM': '!', 'AT': '@', 'HASH': '#', 'PIPE': '|', 'UNDS': '_',
        'PSCREEN': 'PrS', 'SCROLLLOCK': 'ScL', 'PAUSE': 'Pau', 'APPLICATION': 'App',
        'MS_L': 'M←', 'MS_R': 'M→', 'MS_U': 'M↑', 'MS_D': 'M↓',
        'WH_L': 'W←', 'WH_R': 'W→', 'WH_U': 'W↑', 'WH_D': 'W↓',
        'BTN1': 'Lck', 'BTN2': 'Rck', 'BTN3': 'Mck',
        'MPRV': 'Prv', 'MNXT': 'Nxt', 'VOLU': 'V+', 'VOLD': 'V-',
        'MPLY': 'Ply', 'MSTP': 'Stp', 'MUTE': 'Mut',
        'RGB_TOG': 'RGB', 'RGB_MOD': 'Mod', 'RGB_HUI': 'Hue', 'RGB_SAI': 'Sat', 'RGB_VAI': 'Val',
        'AGAIN': 'Redo', 'PASTE': 'Pst', 'COPY': 'Cpy', 'CUT': 'Cut', 'UNDO': 'Und',
        'CW_TOGG': 'CpWd', 'QK_BOOT': 'Boot', 'OU_AUTO': 'OUAt',
    }

    if kc in simplify:
        return simplify[kc]

    if m := re.match(r'F(\d+)', kc):
        return f"F{m.group(1)}"

    return kc[:4] if len(kc) > 4 else kc


def load_layout(vil_file):
    """Load layout from .vil file"""
    with open(vil_file) as f:
        data = json.load(f)
    return data['layout']


def build_layer_display(layout, layer_num):
    """Build the display for a layer, returns list of strings"""
    if layer_num >= len(layout):
        return []

    layer = layout[layer_num]
    layer_name = LAYER_NAMES[layer_num] if layer_num < len(LAYER_NAMES) else f"LAYER {layer_num}"
    layer_color = LAYER_COLORS.get(layer_num, "white")
    active_hand = LAYER_ACTIVE_HAND.get(layer_num, "both")

    left_rows = layer[0:4]
    right_rows = layer[4:8]

    table = Table(show_header=False, box=None, padding=(0, 1))
    for _ in range(12):
        table.add_column(justify="center", width=6)

    for row_idx in range(3):
        left = left_rows[row_idx]
        right = right_rows[row_idx]
        cells = []

        for col, key in enumerate(left):
            label = simplify_keycode(key)
            color = COL_COLORS.get(col, "white")
            if active_hand == "right" and label != "·":
                cells.append(f"[dim]{label}[/dim]")
            elif label == "·":
                cells.append(f"[dim]{label}[/dim]")
            else:
                cells.append(f"[{color} bold]{label}[/{color} bold]")

        cells.extend(["", ""])

        for col, key in enumerate(right):
            label = simplify_keycode(key)
            color = COL_COLORS.get(col + 5, "white")
            if active_hand == "left" and label != "·":
                cells.append(f"[dim]{label}[/dim]")
            elif label == "·":
                cells.append(f"[dim]{label}[/dim]")
            else:
                cells.append(f"[{color} bold]{label}[/{color} bold]")

        table.add_row(*cells)

    # Thumbs
    left_t = [simplify_keycode(k) for k in left_rows[3] if k != -1]
    right_t = [simplify_keycode(k) for k in right_rows[3] if k != -1]

    thumb_cells = ["", ""]
    for t in left_t:
        if t == "·" or active_hand == "right":
            thumb_cells.append(f"[dim]{t}[/dim]")
        else:
            thumb_cells.append(f"[{THUMB_COLOR} bold]{t}[/{THUMB_COLOR} bold]")

    thumb_cells.extend(["", ""])

    for t in right_t:
        if t == "·" or active_hand == "left":
            thumb_cells.append(f"[dim]{t}[/dim]")
        else:
            thumb_cells.append(f"[{THUMB_COLOR} bold]{t}[/{THUMB_COLOR} bold]")

    while len(thumb_cells) < 12:
        thumb_cells.append("")
    table.add_row(*thumb_cells[:12])

    # Layer bar
    layer_bar = []
    for i, name in enumerate(LAYER_NAMES[:8]):
        short = name.split()[0][:4]
        if i == layer_num:
            layer_bar.append(f"[{LAYER_COLORS[i]} bold reverse] {short} [/]")
        else:
            layer_bar.append(f"[dim]{short}[/dim]")

    return (layer_name, layer_color, table, " ".join(layer_bar))


def render_layer(console, layout, layer_num):
    """Render a layer to the console"""
    result = build_layer_display(layout, layer_num)
    if not result:
        return

    layer_name, layer_color, table, bar = result

    console.clear()
    console.print()
    console.print(f"[{layer_color} bold]  {layer_name}[/]", justify="center")
    console.print()
    console.print(table, justify="center")
    console.print()
    console.print(bar, justify="center")
    console.print()
    console.print("[dim]Keys: 0-7 = layers, q = quit | Fingers: [bright_cyan]pinky[/] [bright_magenta]ring[/] [bright_green]middle[/] [bright_yellow]index[/] [bright_blue]thumb[/][/dim]", justify="center")


def interactive_mode(console, vil_file):
    """Interactive mode with keyboard input and hot-reload"""
    layout = load_layout(vil_file)
    file_mtime = os.stat(vil_file).st_mtime
    current_layer = 0

    fd = sys.stdin.fileno()
    old_settings = termios.tcgetattr(fd)

    try:
        tty.setcbreak(fd)
        render_layer(console, layout, current_layer)

        while True:
            # Check for file changes
            try:
                new_mtime = os.stat(vil_file).st_mtime
                if new_mtime > file_mtime:
                    file_mtime = new_mtime
                    layout = load_layout(vil_file)
                    render_layer(console, layout, current_layer)
            except OSError:
                pass

            # Non-blocking input check
            if select.select([sys.stdin], [], [], 0.2)[0]:
                ch = sys.stdin.read(1)
                if ch == 'q' or ch == '\x03':  # q or Ctrl+C
                    break
                elif ch.isdigit() and int(ch) < len(layout):
                    current_layer = int(ch)
                    render_layer(console, layout, current_layer)
                elif ch == 'n' or ch == ' ':  # next layer
                    current_layer = (current_layer + 1) % min(8, len(layout))
                    render_layer(console, layout, current_layer)
                elif ch == 'p':  # previous layer
                    current_layer = (current_layer - 1) % min(8, len(layout))
                    render_layer(console, layout, current_layer)

    except KeyboardInterrupt:
        pass
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old_settings)
        console.clear()


def find_szr35_hidraw():
    """Find the SZR35 Raw HID interface"""
    for i in range(20):
        path = f"/dev/hidraw{i}"
        uevent = f"/sys/class/hidraw/hidraw{i}/device/uevent"
        if not os.path.exists(path):
            continue
        try:
            with open(uevent) as f:
                content = f.read()
                if "SZR35" in content:
                    # Check if this is the raw HID interface (usually input1 or input2)
                    if "input1" in content or "input2" in content:
                        return path, i
        except:
            pass
    return None, None


def hid_mode(console, vil_file):
    """HID mode - reads layer changes directly from keyboard via /dev/hidraw"""
    layout = load_layout(vil_file)
    file_mtime = os.stat(vil_file).st_mtime
    current_layer = 0

    path, num = find_szr35_hidraw()
    if not path:
        console.print("[red]SZR35 keyboard not found![/red]")
        console.print("\nAvailable hidraw devices:")
        for i in range(20):
            uevent = f"/sys/class/hidraw/hidraw{i}/device/uevent"
            if os.path.exists(uevent):
                with open(uevent) as f:
                    for line in f:
                        if "HID_NAME" in line:
                            console.print(f"  hidraw{i}: {line.strip()}")
        console.print("\n[yellow]Try interactive mode instead: python miryoku_trainer.py[/yellow]")
        return

    console.print(f"[green]Found SZR35 at {path}[/green]")

    try:
        # Open for read/write
        fd = os.open(path, os.O_RDWR | os.O_NONBLOCK)

        # Send request for current layer state
        request = bytes([0x00] + [0]*31)
        try:
            os.write(fd, request)
        except Exception:
            pass

        console.print("[green]Connected! Switch layers on your keyboard...[/green]")
        console.print("[dim]Press Ctrl+C to quit[/dim]\n")

        render_layer(console, layout, current_layer)

        while True:
            # Hot reload layout file
            try:
                new_mtime = os.stat(vil_file).st_mtime
                if new_mtime > file_mtime:
                    file_mtime = new_mtime
                    layout = load_layout(vil_file)
                    render_layer(console, layout, current_layer)
            except OSError:
                pass

            # Wait for data with timeout
            r, _, _ = select.select([fd], [], [], 0.2)
            if r:
                try:
                    data = os.read(fd, 64)
                    if data and len(data) >= 2:
                        # MSG_LAYER_STATE = 0x01, byte 1 = layer number
                        if data[0] == 0x01 and 0 <= data[1] <= 7:
                            new_layer = data[1]
                            if new_layer != current_layer:
                                current_layer = new_layer
                                render_layer(console, layout, current_layer)
                except BlockingIOError:
                    pass

    except PermissionError:
        console.print(f"[red]Permission denied on {path}[/red]")
        console.print("[yellow]Run: sudo chmod 666 /dev/hidraw*[/yellow]")
    except KeyboardInterrupt:
        console.clear()
    finally:
        try:
            os.close(fd)
        except:
            pass


def main():
    parser = argparse.ArgumentParser(description='Miryoku Layout Trainer')
    parser.add_argument('--hid', '-H', action='store_true',
                        help='Use HID mode (auto-detect layer from keyboard)')
    parser.add_argument('--file', '-f', type=str, default=str(VIL_FILE),
                        help='Path to .vil file')
    args = parser.parse_args()

    console = Console()
    vil_file = Path(args.file)

    if not vil_file.exists():
        console.print(f"[red]Error: {vil_file} not found[/red]")
        sys.exit(1)

    if args.hid:
        hid_mode(console, vil_file)
    else:
        interactive_mode(console, vil_file)


if __name__ == '__main__':
    main()
