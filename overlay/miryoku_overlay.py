#!/usr/bin/env python3
"""
Miryoku Layout Overlay
A transparent always-on-top overlay that shows your current keyboard layer.
Reads layer state directly from keyboard via /dev/hidraw.
"""

import json
import sys
import os
import threading
import select
from pathlib import Path

from PyQt6.QtWidgets import (
    QApplication, QWidget, QLabel, QVBoxLayout, QHBoxLayout,
    QGridLayout, QFrame
)
from PyQt6.QtCore import Qt, QTimer, QPoint, pyqtSignal, QObject
from PyQt6.QtGui import QFont, QColor, QPalette, QMouseEvent, QKeyEvent

# Find the .vil file relative to the script
SCRIPT_DIR = Path(__file__).parent
VIL_FILE = SCRIPT_DIR.parent / 'layouts' / 'miryoku-kbd-layout.vil'

# Finger colors (matching terminal version)
FINGER_COLORS = {
    0: "#00FFFF",  # Left pinky - cyan
    1: "#FF00FF",  # Left ring - magenta
    2: "#00FF00",  # Left middle - green
    3: "#FFFF00",  # Left index - yellow
    4: "#FFFF00",  # Left index inner - yellow
    5: "#FFFF00",  # Right index inner - yellow
    6: "#FFFF00",  # Right index - yellow
    7: "#00FF00",  # Right middle - green
    8: "#FF00FF",  # Right ring - magenta
    9: "#00FFFF",  # Right pinky - cyan
}

THUMB_COLOR = "#4488FF"
DIM_COLOR = "#666666"

LAYER_COLORS = {
    0: "#FFFFFF",  # Base - white
    1: "#00FFFF",  # Nav - cyan
    2: "#00FF00",  # Mouse - green
    3: "#FF00FF",  # Media - magenta
    4: "#FFFF00",  # Num - yellow
    5: "#FF6666",  # Sym - red
    6: "#6688FF",  # Fun - blue
    7: "#FF8800",  # Button - orange
}

LAYER_NAMES = [
    "BASE",
    "NAV",
    "MOUSE",
    "MEDIA",
    "NUM",
    "SYM",
    "FUN",
    "BUTTON",
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
    import re

    if kc == -1 or kc == "KC_NO":
        return "·"
    if kc == "KC_TRNS":
        return "▽"

    kc = str(kc)
    kc = re.sub(r'^KC_', '', kc)

    # Mod-taps
    if m := re.match(r'(\w+)_T\(KC_(\w+)\)', kc):
        mod, key = m.groups()
        return key

    # Layer-taps
    if m := re.match(r'LT\((\d+),KC_(\w+)\)', kc):
        layer, key = m.groups()
        key_map = {'SPACE': '␣', 'ESCAPE': 'Esc', 'TAB': 'Tab', 'ENTER': '↵',
                   'BSPACE': '⌫', 'DELETE': 'Del', 'Z': 'Z', 'SLASH': '/'}
        return key_map.get(key, key)

    simplify = {
        'SPACE': '␣', 'ESCAPE': 'Esc', 'BSPACE': '⌫', 'DELETE': 'Del',
        'ENTER': '↵', 'TAB': '⇥', 'INSERT': 'Ins', 'HOME': 'Hom',
        'END': 'End', 'PGUP': 'PgU', 'PGDOWN': 'PgD',
        'LEFT': '←', 'RIGHT': '→', 'UP': '↑', 'DOWN': '↓',
        'LSHIFT': '⇧', 'LCTRL': '⌃', 'LALT': '⌥', 'LGUI': '⌘', 'RALT': 'AGr',
        'QUOTE': "'", 'COMMA': ',', 'DOT': '.', 'SLASH': '/', 'SCOLON': ';',
        'LBRACKET': '[', 'RBRACKET': ']', 'BSLASH': '\\', 'GRAVE': '`',
        'EQUAL': '=', 'MINUS': '-',
        'LCBR': '{', 'RCBR': '}', 'LPRN': '(', 'RPRN': ')',
        'AMPR': '&', 'ASTR': '*', 'COLN': ':', 'DLR': '$',
        'PERC': '%', 'CIRC': '^', 'PLUS': '+', 'TILD': '~',
        'EXLM': '!', 'AT': '@', 'HASH': '#', 'PIPE': '|', 'UNDS': '_',
        'PSCREEN': 'PrS', 'SCROLLLOCK': 'ScL', 'PAUSE': 'Pau', 'APPLICATION': 'App',
        'MS_L': '🖰←', 'MS_R': '🖰→', 'MS_U': '🖰↑', 'MS_D': '🖰↓',
        'WH_L': '⟲←', 'WH_R': '⟲→', 'WH_U': '⟲↑', 'WH_D': '⟲↓',
        'BTN1': '🖰L', 'BTN2': '🖰R', 'BTN3': '🖰M',
        'MPRV': '⏮', 'MNXT': '⏭', 'VOLU': '🔊', 'VOLD': '🔉',
        'MPLY': '⏯', 'MSTP': '⏹', 'MUTE': '🔇',
        'RGB_TOG': 'RGB', 'RGB_MOD': 'Mod', 'RGB_HUI': 'Hue', 'RGB_SAI': 'Sat', 'RGB_VAI': 'Val',
        'AGAIN': 'Redo', 'PASTE': 'Pst', 'COPY': 'Cpy', 'CUT': 'Cut', 'UNDO': 'Und',
        'CW_TOGG': 'CpWd', 'QK_BOOT': 'Boot', 'OU_AUTO': 'OUAt',
    }

    if kc in simplify:
        return simplify[kc]

    if m := re.match(r'F(\d+)', kc):
        return f"F{m.group(1)}"

    return kc[:3] if len(kc) > 3 else kc


class KeyLabel(QLabel):
    """A styled key label"""
    def __init__(self, text="", color="#FFFFFF", dimmed=False):
        super().__init__(text)
        self.base_color = color
        self.dimmed = dimmed
        self.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.setMinimumSize(45, 40)
        self.update_style()

    def update_style(self):
        color = DIM_COLOR if self.dimmed else self.base_color
        opacity = "0.4" if self.dimmed else "0.9"
        self.setStyleSheet(f"""
            QLabel {{
                background-color: rgba(30, 30, 30, {opacity});
                color: {color};
                border: 1px solid {color};
                border-radius: 6px;
                font-size: 14px;
                font-weight: bold;
                padding: 4px;
            }}
        """)

    def set_dimmed(self, dimmed):
        self.dimmed = dimmed
        self.update_style()

    def set_text_and_color(self, text, color, dimmed=False):
        self.setText(text)
        self.base_color = color
        self.dimmed = dimmed
        self.update_style()


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


class VialHIDMonitor(QObject):
    """Monitor keyboard layer state via direct /dev/hidraw access"""
    layer_changed = pyqtSignal(int)

    def __init__(self):
        super().__init__()
        self.running = False
        self.thread = None
        self.fd = None
        self.current_layer = 0

    def start(self):
        self.running = True
        self.thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self.thread.start()

    def stop(self):
        self.running = False
        if self.fd is not None:
            try:
                os.close(self.fd)
            except:
                pass

    def _monitor_loop(self):
        path, num = find_szr35_hidraw()
        if not path:
            print("SZR35 keyboard not found!")
            print("Available hidraw devices:")
            for i in range(20):
                uevent = f"/sys/class/hidraw/hidraw{i}/device/uevent"
                if os.path.exists(uevent):
                    with open(uevent) as f:
                        for line in f:
                            if "HID_NAME" in line:
                                print(f"  hidraw{i}: {line.strip()}")
            print("\nYou can still manually switch layers with keys 0-7")
            return

        print(f"Found SZR35 at {path}")

        try:
            # Open for read/write
            self.fd = os.open(path, os.O_RDWR | os.O_NONBLOCK)

            # Send request for current layer state
            request = bytes([0x00] + [0]*31)
            try:
                os.write(self.fd, request)
            except Exception:
                pass

            print("Connected! Monitoring for layer changes...")

            while self.running:
                # Wait for data with timeout
                r, _, _ = select.select([self.fd], [], [], 0.2)
                if r:
                    try:
                        data = os.read(self.fd, 64)
                        if data and len(data) >= 2:
                            # MSG_LAYER_STATE = 0x01, byte 1 = layer number
                            if data[0] == 0x01 and 0 <= data[1] <= 7:
                                new_layer = data[1]
                                if new_layer != self.current_layer:
                                    self.current_layer = new_layer
                                    self.layer_changed.emit(new_layer)
                    except BlockingIOError:
                        pass

        except PermissionError:
            print(f"Permission denied on {path}")
            print("Run: sudo chmod 666 /dev/hidraw*")
        except Exception as e:
            print(f"HID error: {e}")
        finally:
            if self.fd is not None:
                try:
                    os.close(self.fd)
                except:
                    pass


class KeyboardMonitor(QObject):
    """Monitor keyboard events via evdev to detect layer changes based on key signatures"""
    layer_changed = pyqtSignal(int)

    def __init__(self, device_filter=None):
        super().__init__()
        self.running = False
        self.thread = None
        self.device_filter = device_filter
        self.current_layer = 0
        self.last_layer_change = 0
        self.held_keys = set()

        # Keys that indicate specific layers (evdev key codes)
        self.layer_signatures = {
            1: {103, 108, 105, 106, 102, 107, 104, 109},  # NAV: arrows, home/end/pgup/pgdn
            2: set(),  # MOUSE - can't detect
            3: {165, 163, 164, 166, 113, 114, 115},  # MEDIA: media keys
            4: {71, 72, 73, 75, 76, 77, 79, 80, 81, 82, 83},  # NUM: keypad
            5: set(),  # SYM - can't detect (shifted chars)
            6: {59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 87, 88},  # FUN: F1-F12
        }

        self.modifier_codes = {29, 97, 56, 100, 42, 54, 125, 126}

    def start(self):
        self.running = True
        self.thread = threading.Thread(target=self._monitor_loop, daemon=True)
        self.thread.start()

    def stop(self):
        self.running = False

    def _monitor_loop(self):
        try:
            import evdev
            from evdev import ecodes
        except ImportError:
            print("evdev not installed. Layer detection disabled.")
            return

        devices = []
        for path in evdev.list_devices():
            dev = evdev.InputDevice(path)
            if ecodes.EV_KEY in dev.capabilities():
                if self.device_filter:
                    if self.device_filter.lower() in dev.name.lower():
                        devices.append(dev)
                        print(f"Monitoring: {dev.name}")
                else:
                    devices.append(dev)

        if not devices:
            print(f"No devices matching '{self.device_filter}' found." if self.device_filter else "No keyboard devices found.")
            return

        import select

        while self.running:
            r, w, x = select.select(devices, [], [], 0.1)
            for dev in r:
                try:
                    for event in dev.read():
                        if event.type == ecodes.EV_KEY:
                            self._handle_key_event(event.code, event.value)
                except Exception:
                    pass

    def _handle_key_event(self, code, value):
        now = time.time()

        if value == 1:
            self.held_keys.add(code)
        elif value == 0:
            self.held_keys.discard(code)

        if code in self.modifier_codes:
            return

        detected_layer = 0
        for layer, sig_keys in self.layer_signatures.items():
            if self.held_keys & sig_keys:
                detected_layer = layer
                break

        if detected_layer != self.current_layer:
            if detected_layer > 0 or (now - self.last_layer_change > 0.3):
                self.current_layer = detected_layer
                self.last_layer_change = now
                self.layer_changed.emit(detected_layer)

        if value == 0 and self.current_layer != 0:
            has_sig_key = any(self.held_keys & sig for sig in self.layer_signatures.values())
            if not has_sig_key and (now - self.last_layer_change > 0.1):
                self.current_layer = 0
                self.last_layer_change = now
                self.layer_changed.emit(0)


class MiryokuOverlay(QWidget):
    def __init__(self, vil_file, device_filter=None, use_hid=True, use_evdev=True):
        super().__init__()
        self.vil_file = vil_file
        self.layout_data = self.load_layout()
        self.current_layer = 0
        self.file_mtime = os.stat(vil_file).st_mtime
        self.drag_position = None
        self.monitors = []

        self.init_ui()
        self.setup_hotreload()

        # Try HID first (most accurate), fall back to evdev
        if use_hid:
            hid_monitor = VialHIDMonitor()
            hid_monitor.layer_changed.connect(self.set_layer)
            hid_monitor.start()
            self.monitors.append(hid_monitor)

        if use_evdev:
            evdev_monitor = KeyboardMonitor(device_filter=device_filter)
            evdev_monitor.layer_changed.connect(self.set_layer)
            evdev_monitor.start()
            self.monitors.append(evdev_monitor)

    def load_layout(self):
        with open(self.vil_file) as f:
            data = json.load(f)
        return data['layout']

    def init_ui(self):
        self.setWindowTitle("Miryoku Overlay")

        self.setWindowFlags(
            Qt.WindowType.FramelessWindowHint |
            Qt.WindowType.WindowStaysOnTopHint |
            Qt.WindowType.Tool |
            Qt.WindowType.X11BypassWindowManagerHint
        )
        self.setAttribute(Qt.WidgetAttribute.WA_TranslucentBackground)
        self.setAttribute(Qt.WidgetAttribute.WA_ShowWithoutActivating)

        main_layout = QVBoxLayout(self)
        main_layout.setContentsMargins(10, 10, 10, 10)

        container = QFrame()
        container.setStyleSheet("""
            QFrame {
                background-color: rgba(20, 20, 20, 0.85);
                border-radius: 12px;
            }
        """)

        container_layout = QVBoxLayout(container)
        container_layout.setSpacing(8)

        self.layer_label = QLabel("BASE")
        self.layer_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        self.layer_label.setFont(QFont("monospace", 18, QFont.Weight.Bold))
        self.layer_label.setStyleSheet("color: #FFFFFF; padding: 5px;")
        container_layout.addWidget(self.layer_label)

        keyboard_widget = QWidget()
        keyboard_layout = QVBoxLayout(keyboard_widget)
        keyboard_layout.setSpacing(4)

        self.key_labels = []

        for row in range(4):
            row_widget = QWidget()
            row_layout = QHBoxLayout(row_widget)
            row_layout.setSpacing(4)
            row_layout.setContentsMargins(0, 0, 0, 0)

            row_labels = []

            if row < 3:
                for col in range(5):
                    label = KeyLabel()
                    row_layout.addWidget(label)
                    row_labels.append(label)

                spacer = QLabel()
                spacer.setFixedWidth(30)
                row_layout.addWidget(spacer)

                for col in range(5):
                    label = KeyLabel()
                    row_layout.addWidget(label)
                    row_labels.append(label)
            else:
                row_layout.addStretch()
                for col in range(3):
                    label = KeyLabel()
                    row_layout.addWidget(label)
                    row_labels.append(label)

                spacer = QLabel()
                spacer.setFixedWidth(30)
                row_layout.addWidget(spacer)

                for col in range(3):
                    label = KeyLabel()
                    row_layout.addWidget(label)
                    row_labels.append(label)
                row_layout.addStretch()

            keyboard_layout.addWidget(row_widget)
            self.key_labels.append(row_labels)

        container_layout.addWidget(keyboard_widget)

        layer_bar = QWidget()
        layer_bar_layout = QHBoxLayout(layer_bar)
        layer_bar_layout.setSpacing(4)

        self.layer_buttons = []
        for i, name in enumerate(LAYER_NAMES):
            btn = QLabel(name[:3])
            btn.setAlignment(Qt.AlignmentFlag.AlignCenter)
            btn.setFixedSize(40, 25)
            btn.setStyleSheet(f"""
                QLabel {{
                    background-color: rgba(50, 50, 50, 0.8);
                    color: {DIM_COLOR};
                    border-radius: 4px;
                    font-size: 11px;
                }}
            """)
            layer_bar_layout.addWidget(btn)
            self.layer_buttons.append(btn)

        container_layout.addWidget(layer_bar)

        help_label = QLabel("0-7: layers | Drag to move | Q: quit")
        help_label.setAlignment(Qt.AlignmentFlag.AlignCenter)
        help_label.setStyleSheet("color: #666666; font-size: 10px; padding: 2px;")
        container_layout.addWidget(help_label)

        main_layout.addWidget(container)

        self.update_display()

        self.adjustSize()
        screen = QApplication.primaryScreen().geometry()
        self.move(
            (screen.width() - self.width()) // 2,
            screen.height() - self.height() - 50
        )

    def setup_hotreload(self):
        self.reload_timer = QTimer(self)
        self.reload_timer.timeout.connect(self.check_file_changes)
        self.reload_timer.start(500)

    def check_file_changes(self):
        try:
            new_mtime = os.stat(self.vil_file).st_mtime
            if new_mtime > self.file_mtime:
                self.file_mtime = new_mtime
                self.layout_data = self.load_layout()
                self.update_display()
        except OSError:
            pass

    def update_display(self):
        if self.current_layer >= len(self.layout_data):
            return

        layer = self.layout_data[self.current_layer]
        layer_color = LAYER_COLORS.get(self.current_layer, "#FFFFFF")
        active_hand = LAYER_ACTIVE_HAND.get(self.current_layer, "both")

        self.layer_label.setText(LAYER_NAMES[self.current_layer])
        self.layer_label.setStyleSheet(f"color: {layer_color}; padding: 5px;")

        for i, btn in enumerate(self.layer_buttons):
            if i == self.current_layer:
                btn.setStyleSheet(f"""
                    QLabel {{
                        background-color: {LAYER_COLORS.get(i, '#FFFFFF')};
                        color: #000000;
                        border-radius: 4px;
                        font-size: 11px;
                        font-weight: bold;
                    }}
                """)
            else:
                btn.setStyleSheet(f"""
                    QLabel {{
                        background-color: rgba(50, 50, 50, 0.8);
                        color: {DIM_COLOR};
                        border-radius: 4px;
                        font-size: 11px;
                    }}
                """)

        left_rows = layer[0:4]
        right_rows = layer[4:8]

        for row_idx in range(3):
            left = left_rows[row_idx]
            right = right_rows[row_idx]

            for col in range(5):
                label = self.key_labels[row_idx][col]
                text = simplify_keycode(left[col])
                color = FINGER_COLORS.get(col, "#FFFFFF")
                dimmed = (active_hand == "right" and text != "·") or text == "·"
                label.set_text_and_color(text, color, dimmed)

            for col in range(5):
                label = self.key_labels[row_idx][col + 5]
                text = simplify_keycode(right[col])
                color = FINGER_COLORS.get(col + 5, "#FFFFFF")
                dimmed = (active_hand == "left" and text != "·") or text == "·"
                label.set_text_and_color(text, color, dimmed)

        left_thumbs = [k for k in left_rows[3] if k != -1]
        right_thumbs = [k for k in right_rows[3] if k != -1]

        for i, key in enumerate(left_thumbs):
            if i < len(self.key_labels[3]):
                text = simplify_keycode(key)
                dimmed = (active_hand == "right" and text != "·") or text == "·"
                self.key_labels[3][i].set_text_and_color(text, THUMB_COLOR, dimmed)

        for i, key in enumerate(right_thumbs):
            idx = i + 3
            if idx < len(self.key_labels[3]):
                text = simplify_keycode(key)
                dimmed = (active_hand == "left" and text != "·") or text == "·"
                self.key_labels[3][idx].set_text_and_color(text, THUMB_COLOR, dimmed)

    def set_layer(self, layer_num):
        if 0 <= layer_num < len(self.layout_data):
            if layer_num != self.current_layer:
                self.current_layer = layer_num
                self.update_display()

    def keyPressEvent(self, event: QKeyEvent):
        key = event.key()

        if Qt.Key.Key_0 <= key <= Qt.Key.Key_7:
            self.set_layer(key - Qt.Key.Key_0)
        elif key == Qt.Key.Key_Q:
            self.close()
        elif key == Qt.Key.Key_N:
            self.set_layer((self.current_layer + 1) % min(8, len(self.layout_data)))
        elif key == Qt.Key.Key_P:
            self.set_layer((self.current_layer - 1) % min(8, len(self.layout_data)))

    def mousePressEvent(self, event: QMouseEvent):
        if event.button() == Qt.MouseButton.LeftButton:
            self.drag_position = event.globalPosition().toPoint() - self.frameGeometry().topLeft()
            event.accept()

    def mouseMoveEvent(self, event: QMouseEvent):
        if event.buttons() == Qt.MouseButton.LeftButton and self.drag_position:
            self.move(event.globalPosition().toPoint() - self.drag_position)
            event.accept()

    def mouseReleaseEvent(self, event: QMouseEvent):
        self.drag_position = None

    def closeEvent(self, event):
        for monitor in self.monitors:
            monitor.stop()
        event.accept()


def main():
    import argparse

    parser = argparse.ArgumentParser(description='Miryoku Layout Overlay')
    parser.add_argument('--file', '-f', type=str, default=str(VIL_FILE),
                        help='Path to .vil file')
    parser.add_argument('--device', '-d', type=str, default=None,
                        help='Filter to keyboard device containing this name (for evdev). Auto-detects if not specified.')
    parser.add_argument('--no-hid', action='store_true',
                        help='Disable HID layer detection')
    parser.add_argument('--no-evdev', action='store_true',
                        help='Disable evdev key detection')
    args = parser.parse_args()

    vil_file = Path(args.file)
    if not vil_file.exists():
        print(f"Error: {vil_file} not found")
        sys.exit(1)

    app = QApplication(sys.argv)
    app.setApplicationName("Miryoku Overlay")

    app.setStyle("Fusion")
    palette = QPalette()
    palette.setColor(QPalette.ColorRole.Window, QColor(30, 30, 30))
    palette.setColor(QPalette.ColorRole.WindowText, QColor(255, 255, 255))
    app.setPalette(palette)

    overlay = MiryokuOverlay(
        vil_file,
        device_filter=args.device,
        use_hid=not args.no_hid,
        use_evdev=not args.no_evdev
    )
    overlay.show()

    sys.exit(app.exec())


if __name__ == '__main__':
    main()
