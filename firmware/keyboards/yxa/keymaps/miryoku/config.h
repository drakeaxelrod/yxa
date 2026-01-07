#pragma once

// VIAL configuration
#define VIAL_KEYBOARD_UID {0x59, 0x58, 0x41, 0x4B, 0x42, 0x44, 0x01, 0x00}
#define VIAL_UNLOCK_COMBO_ROWS {0, 4}
#define VIAL_UNLOCK_COMBO_COLS {4, 0}

// 10 layers for full Miryoku support
#define DYNAMIC_KEYMAP_LAYER_COUNT 10

// ============================================================================
// Home Row Mods - Tap-Hold Configuration
// ============================================================================

// Base tapping term (ms) - time to hold before it becomes a hold action
#define TAPPING_TERM 200

// Home row mods tapping term - longer to avoid accidental holds during fast typing
#define TAPPING_TERM_MOD_TAP 220

// Layer key tapping term - for responsive layer switching on thumb keys
#define TAPPING_TERM_LAYER 200

// Per-key tapping term: uses get_tapping_term() in yxa_features.c
#define TAPPING_TERM_PER_KEY

// QUICK_TAP_TERM: Double-tap within this time = always tap (good for "ff", "ss")
#define QUICK_TAP_TERM 120

// PERMISSIVE_HOLD: If you press mod-tap key, then press AND RELEASE another
// key before tapping term expires, treat as a hold. This makes modifiers
// work faster when you're deliberately using them.
// Without this: A(down) -> B(down) -> B(up) -> A(up) = "ab"
// With this:    A(down) -> B(down) -> B(up) -> A(up) = "Shift+B" (if A is shift)
#define PERMISSIVE_HOLD

// Enable per-key permissive hold for fine control
#define PERMISSIVE_HOLD_PER_KEY

// HOLD_ON_OTHER_KEY_PRESS: Immediately activate hold when another key is pressed
// We use per-key version for bilateral combinations (mods only trigger with opposite hand)
#define HOLD_ON_OTHER_KEY_PRESS_PER_KEY

// RETRO_TAPPING: If you hold a key past tapping term but don't press any other
// key, send the tap keycode on release. Can be useful but also causes misfires.
// #define RETRO_TAPPING

// IGNORE_MOD_TAP_INTERRUPT: Deprecated in favor of HOLD_ON_OTHER_KEY_PRESS
// Don't use this.

// ============================================================================
// RGB Layer Indication
// ============================================================================

// Full brightness for layer colors (0-255)
#define RGB_LAYER_BRIGHTNESS 255

// Dim brightness for TAP layer indicator (0-255)
#define RGB_TAP_BRIGHTNESS 100
