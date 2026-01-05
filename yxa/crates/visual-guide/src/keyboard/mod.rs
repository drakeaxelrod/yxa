//! Keyboard communication and layout handling

mod hid;
mod layout;

pub use hid::SyncHidMonitor;
pub use layout::{
    active_hand, finger_color, layer_color, layer_name, load_layout, parse_key_label,
    simplify_keycode, ActiveHand, HoldType, KeyLabel, Layer, THUMB_COLOR,
};
