use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

/// A keycode can be a string like "KC_A" or an integer like -1
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum Keycode {
    String(String),
    Int(i32),
}

impl Keycode {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Keycode::String(s) => Some(s),
            Keycode::Int(_) => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Keycode::String(s) => s == "KC_NO",
            Keycode::Int(n) => *n == -1,
        }
    }
}

/// A row is 5 keys
pub type Row = Vec<Keycode>;

/// A layer is 8 rows: 4 left hand (0-3), 4 right hand (4-7)
/// Row 3 and 7 are thumb rows with -1 padding
pub type Layer = Vec<Row>;

/// The complete layout with all layers
#[derive(Debug, Deserialize)]
pub struct VilFile {
    pub layout: Vec<Layer>,
}

/// Finger colors for visual learning
pub static FINGER_COLORS: LazyLock<HashMap<usize, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (0, "cyan"),      // Left pinky
        (1, "magenta"),   // Left ring
        (2, "green"),     // Left middle
        (3, "yellow"),    // Left index
        (4, "yellow"),    // Left index inner
        (5, "yellow"),    // Right index inner
        (6, "yellow"),    // Right index
        (7, "green"),     // Right middle
        (8, "magenta"),   // Right ring
        (9, "cyan"),      // Right pinky
    ])
});

pub const THUMB_COLOR: &str = "blue";

pub static LAYER_COLORS: LazyLock<HashMap<usize, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        (0, "white"),
        (1, "cyan"),
        (2, "green"),
        (3, "magenta"),
        (4, "yellow"),
        (5, "red"),
        (6, "blue"),
        (7, "light_yellow"),
    ])
});

pub const LAYER_NAMES: &[&str] = &[
    "BASE (Colemak-DH)",
    "NAV ← ↓ ↑ →",
    "MOUSE",
    "MEDIA ♫",
    "NUM 123",
    "SYM !@#",
    "FUN F1-F12",
    "BUTTON",
    "EXTRA",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveHand {
    Both,
    Left,
    Right,
}

pub static LAYER_ACTIVE_HAND: LazyLock<HashMap<usize, ActiveHand>> = LazyLock::new(|| {
    HashMap::from([
        (0, ActiveHand::Both),
        (1, ActiveHand::Right),
        (2, ActiveHand::Right),
        (3, ActiveHand::Right),
        (4, ActiveHand::Left),
        (5, ActiveHand::Left),
        (6, ActiveHand::Left),
        (7, ActiveHand::Both),
    ])
});

static MOD_TAP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\w+)_T\(KC_(\w+)\)$").unwrap());
static LAYER_TAP_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^LT\((\d+),KC_(\w+)\)$").unwrap());
static FKEY_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^F(\d+)$").unwrap());

/// Simplify a QMK keycode to a readable label
pub fn simplify_keycode(kc: &Keycode) -> String {
    match kc {
        Keycode::Int(-1) => "".to_string(),
        Keycode::Int(_) => "".to_string(),
        Keycode::String(s) if s == "KC_NO" => "".to_string(),
        Keycode::String(s) if s == "KC_TRNS" => "".to_string(),
        Keycode::String(s) => simplify_keycode_str(s),
    }
}

fn simplify_keycode_str(kc: &str) -> String {
    // Remove KC_ prefix
    let kc = kc.strip_prefix("KC_").unwrap_or(kc);

    // Mod-taps: LGUI_T(KC_A) -> A/Gui
    if let Some(caps) = MOD_TAP_RE.captures(kc) {
        let modifier = &caps[1];
        let key = &caps[2];
        let mod_short = match modifier {
            "LGUI" => "Meta",
            "LALT" => "Alt",
            "LCTL" => "Ctrl",
            "LSFT" => "Shift",
            "RALT" => "AltGr",
            _ => modifier,
        };
        return format!("{}/{}", key, mod_short);
    }

    // Layer-taps: LT(1,KC_SPACE) -> Spc/L1
    if let Some(caps) = LAYER_TAP_RE.captures(kc) {
        let layer = &caps[1];
        let key = &caps[2];
        let key_short = match key {
            "SPACE" => "Spc",
            "ESCAPE" => "Esc",
            "TAB" => "Tab",
            "ENTER" => "Ent",
            "BSPACE" => "Bsp",
            "DELETE" => "Del",
            "Z" => "Z",
            "SLASH" => "/",
            _ => key,
        };
        return format!("{}/L{}", key_short, layer);
    }

    // Simple keycode mappings (using Nerd Font symbols where appropriate)
    let simplified = match kc {
        "SPACE" => "󱁐",        // nf-md-keyboard_space
        "ESCAPE" => "󱊷",       // nf-md-keyboard_esc
        "BSPACE" => "󰁮",       // nf-md-backspace
        "DELETE" => "󰹾",       // nf-md-delete
        "ENTER" => "󰌑",        // nf-md-keyboard_return
        "TAB" => "",          // nf-md-keyboard_tab
        "INSERT" => "Ins",
        "HOME" => "Home",          // nf-fa-home
        "END" => "End",
        "PGUP" => "PgUp",         // nf-md-chevron_double_up
        "PGDOWN" => "PgDn",       // nf-md-chevron_double_down
        "LEFT" => "",          // nf-fa-arrow_left
        "RIGHT" => "",         // nf-fa-arrow_right
        "UP" => "",            // nf-fa-arrow_up
        "DOWN" => "",          // nf-fa-arrow_down
        "LSHIFT" => "󰘶",       // nf-md-apple_keyboard_shift
        "LCTRL" => "Ctrl",
        "LALT" => "Alt",
        "LGUI" => "Meta",         // nf-md-apple_keyboard_command
        "RALT" => "Alt Gr",
        "QUOTE" => "'",
        "COMMA" => ",",
        "DOT" => ".",
        "SLASH" => "/",
        "SCOLON" => ";",
        "LBRACKET" => "[",
        "RBRACKET" => "]",
        "BSLASH" => "\\",
        "GRAVE" => "`",
        "EQUAL" => "=",
        "MINUS" => "-",
        "LCBR" => "{",
        "RCBR" => "}",
        "LPRN" => "(",
        "RPRN" => ")",
        "AMPR" => "&",
        "ASTR" => "*",
        "COLN" => ":",
        "DLR" => "$",
        "PERC" => "%",
        "CIRC" => "^",
        "PLUS" => "+",
        "TILD" => "~",
        "EXLM" => "!",
        "AT" => "@",
        "HASH" => "#",
        "PIPE" => "|",
        "UNDS" => "_",
        "PSCREEN" => "",      // nf-md-monitor_screenshot
        "SCROLLLOCK" => "ScrLk",
        "PAUSE" => "",        // nf-md-pause
        "APPLICATION" => "󰍜",  // nf-md-menu
        "MS_L" => " 󰍽",         // nf-md-mouse (left arrow implied)
        "MS_R" => "󰍽 ",
        "MS_U" => "󰍽 ",
        "MS_D" => " 󰍽",
        "WH_L" => "WH_L",         // nf-md-mouse_scroll_left (approximate)
        "WH_R" => "WH_R",         // nf-md-mouse_scroll_right
        "WH_U" => "󱕑",         // nf-md-mouse_scroll_up (scroll wheel)
        "WH_D" => "󱕐",         // nf-md-mouse_scroll_down
        "BTN1" => " 󰍽",         // nf-md-mouse (left click)
        "BTN2" => "󰍽 ",         // nf-md-mouse (right click)
        "BTN3" => "󰍽",         // nf-md-mouse (middle click)
        "MPRV" => "󰒮",         // nf-md-skip_previous
        "MNXT" => "󰒭",         // nf-md-skip_next
        "VOLU" => "󰕾",         // nf-md-volume_high
        "VOLD" => "󰖀",         // nf-md-volume_medium
        "MPLY" => "󰐊",         // nf-md-play
        "MSTP" => "󰓛",         // nf-md-stop
        "MUTE" => "󰝟",         // nf-md-volume_off
        "RGB_TOG" => "󰌬",      // nf-md-led_on
        "RGB_MOD" => "󰔎",      // nf-md-palette
        "RGB_HUI" => "󰏘",      // nf-md-palette (hue)
        "RGB_SAI" => "󰏘",      // nf-md-palette (saturation)
        "RGB_VAI" => "󰃟",      // nf-md-brightness_6
        "AGAIN" => "󰑎",        // nf-md-redo
        "PASTE" => "󰆒",        // nf-md-content_paste
        "COPY" => "󰆏",         // nf-md-content_copy
        "CUT" => "󰆐",          // nf-md-content_cut
        "UNDO" => "󰕌",         // nf-md-undo
        "CW_TOGG" => "󰬶",      // nf-md-caps_lock (caps word)
        "QK_BOOT" => "󰑓",      // nf-md-restart
        "OU_AUTO" => "󰒋",      // nf-md-usb
        _ => {
            // Check for function keys
            if let Some(caps) = FKEY_RE.captures(kc) {
                return format!("F{}", &caps[1]);
            }
            // Truncate if too long
            if kc.len() > 4 {
                return kc[..4].to_string();
            }
            return kc.to_string();
        }
    };

    simplified.to_string()
}

/// Extract modifier from mod-tap keycode for overlay display
pub fn get_mod_from_keycode(kc: &Keycode) -> Option<&'static str> {
    let s = kc.as_str()?;
    let s = s.strip_prefix("KC_").unwrap_or(s);

    if let Some(caps) = MOD_TAP_RE.captures(s) {
        let modifier = &caps[1];
        return Some(match modifier {
            "LGUI" | "RGUI" => "Meta",
            "LALT" => "Alt",
            "RALT" => "AltGr",
            "LCTL" | "RCTL" => "Ctrl",
            "LSFT" | "RSFT" => "Shift",
            _ => return None,
        });
    }
    None
}

/// Key label info for smart display
#[derive(Debug, Clone)]
pub struct KeyLabel {
    pub tap: String,
    pub hold: Option<HoldType>,
}

/// Parse a keycode into tap and hold parts for smart display
pub fn parse_key_label(kc: &Keycode) -> KeyLabel {
    match kc {
        Keycode::Int(-1) => KeyLabel { tap: "󰝦".to_string(), hold: None },  // nf-md-circle_outline
        Keycode::Int(_) => KeyLabel { tap: "󰝦".to_string(), hold: None },
        Keycode::String(s) if s == "KC_NO" => KeyLabel { tap: "󰝦".to_string(), hold: None },
        Keycode::String(s) if s == "KC_TRNS" => KeyLabel { tap: "󰝦".to_string(), hold: None },
        Keycode::String(s) => parse_key_label_str(s),
    }
}

/// Hold type for distinguishing modifiers from layers
#[derive(Debug, Clone)]
pub enum HoldType {
    Modifier(String),
    Layer(usize, String), // layer index and short name
}

fn parse_key_label_str(kc: &str) -> KeyLabel {
    let kc = kc.strip_prefix("KC_").unwrap_or(kc);

    // Mod-taps: LGUI_T(KC_A) -> tap: A, hold: gui
    if let Some(caps) = MOD_TAP_RE.captures(kc) {
        let modifier = &caps[1];
        let key = &caps[2];
        let mod_name = match modifier {
            "LGUI" | "RGUI" => "Meta",
            "LALT" => "Alt",
            "RALT" => "AltGr",
            "LCTL" | "RCTL" => "Ctrl",
            "LSFT" | "RSFT" => "Shift",
            _ => modifier,
        };
        // Simplify the key part (e.g., DOT -> .)
        let tap = simplify_keycode_str(&format!("KC_{}", key));
        return KeyLabel {
            tap,
            hold: Some(HoldType::Modifier(mod_name.to_string())),
        };
    }

    // Layer-taps: LT(1,KC_SPACE) -> tap: Spc, hold: layer info
    if let Some(caps) = LAYER_TAP_RE.captures(kc) {
        let layer: usize = caps[1].parse().unwrap_or(0);
        let key = &caps[2];
        // Simplify the key part using the same logic as other keycodes
        let tap = simplify_keycode_str(&format!("KC_{}", key));
        // Short layer names for display (lowercase)
        let layer_short = match layer {
            0 => "bas",
            1 => "nav",
            2 => "mou",
            3 => "med",
            4 => "num",
            5 => "sym",
            6 => "fun",
            7 => "btn",
            8 => "ext",
            _ => "?",
        };
        return KeyLabel {
            tap,
            hold: Some(HoldType::Layer(layer, layer_short.to_string())),
        };
    }

    // Simple keycode - just tap, no hold
    let tap = simplify_keycode_str(&format!("KC_{}", kc));
    KeyLabel { tap, hold: None }
}

/// Load layout from a .vil file
pub fn load_layout(path: &Path) -> Result<Vec<Layer>> {
    let content = fs::read_to_string(path)?;
    let vil: VilFile = serde_json::from_str(&content)?;
    Ok(vil.layout)
}

/// Get layer name by index
pub fn layer_name(idx: usize) -> &'static str {
    LAYER_NAMES.get(idx).copied().unwrap_or("LAYER")
}

/// Get layer color by index
pub fn layer_color(idx: usize) -> &'static str {
    LAYER_COLORS.get(&idx).copied().unwrap_or("white")
}

/// Get active hand for a layer
pub fn active_hand(layer_idx: usize) -> ActiveHand {
    *LAYER_ACTIVE_HAND.get(&layer_idx).unwrap_or(&ActiveHand::Both)
}

/// Get finger color for a column
pub fn finger_color(col: usize) -> &'static str {
    FINGER_COLORS.get(&col).copied().unwrap_or("white")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplify_basic() {
        assert_eq!(simplify_keycode(&Keycode::String("KC_A".into())), "A");
        assert_eq!(simplify_keycode(&Keycode::String("KC_SPACE".into())), "Spc");
        assert_eq!(simplify_keycode(&Keycode::Int(-1)), "·");
    }

    #[test]
    fn test_simplify_mod_tap() {
        assert_eq!(
            simplify_keycode(&Keycode::String("LGUI_T(KC_A)".into())),
            "A/Gui"
        );
    }

    #[test]
    fn test_simplify_layer_tap() {
        assert_eq!(
            simplify_keycode(&Keycode::String("LT(1,KC_SPACE)".into())),
            "Spc/L1"
        );
    }
}
