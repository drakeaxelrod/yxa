// Yxa Keyboard - Custom Features
// SPDX-License-Identifier: GPL-2.0-or-later

#include QMK_KEYBOARD_H
#include "raw_hid.h"

// Message types for HID protocol
#define MSG_REQUEST_STATE   0x00  // Host -> Keyboard: Request full state
#define MSG_LAYER_STATE     0x01  // Keyboard -> Host: Layer changed
#define MSG_KEY_PRESS       0x02  // Keyboard -> Host: Key pressed
#define MSG_KEY_RELEASE     0x03  // Keyboard -> Host: Key released
#define MSG_CAPS_WORD_STATE 0x04  // Keyboard -> Host: Caps Word state
#define MSG_MODIFIER_STATE  0x05  // Keyboard -> Host: Modifier state
#define MSG_HEARTBEAT       0x06  // Host -> Keyboard: Connection check
#define MSG_FULL_STATE      0x07  // Keyboard -> Host: Full state response
#define MSG_KEY_BATCH       0x08  // Keyboard -> Host: Batched key events

#ifndef RAW_EPSIZE
#define RAW_EPSIZE 32
#endif

// State tracking
static uint8_t last_broadcast_layer = 255;
static bool last_caps_word_state = false;
static uint8_t last_modifier_state = 0;

// Event batching for rapid keypresses
#define MAX_BATCH_EVENTS 8
static uint8_t event_batch[MAX_BATCH_EVENTS * 3];  // type, row, col per event
static uint8_t batch_count = 0;
static uint16_t last_batch_time = 0;
#define BATCH_TIMEOUT_MS 1  // Flush batch after 1ms (reduced for responsiveness)

// Track pressed keys to avoid duplicate events
#define MAX_PRESSED_KEYS 10
static uint8_t pressed_keys[MAX_PRESSED_KEYS][2];  // row, col pairs
static uint8_t pressed_key_count = 0;

// Get effective layer (combines default layer with momentary layers)
static uint8_t get_effective_layer(void) {
    layer_state_t effective = layer_state | default_layer_state;
    return get_highest_layer(effective);
}

// Get current modifier state as a bitmask
static uint8_t get_modifier_state(void) {
    return get_mods() | get_oneshot_mods();
}

// Send batched events if any are pending
static void flush_event_batch(void) {
    if (batch_count > 0) {
        uint8_t data[RAW_EPSIZE] = {0};
        data[0] = MSG_KEY_BATCH;
        data[1] = batch_count;
        // Copy events: each event is 3 bytes (type, row, col)
        for (uint8_t i = 0; i < batch_count && i < MAX_BATCH_EVENTS; i++) {
            data[2 + i * 3] = event_batch[i * 3];      // type
            data[2 + i * 3 + 1] = event_batch[i * 3 + 1]; // row
            data[2 + i * 3 + 2] = event_batch[i * 3 + 2]; // col
        }
        raw_hid_send(data, RAW_EPSIZE);
        batch_count = 0;
    }
}

// Check if a key is in our pressed tracking array
static bool is_key_tracked(uint8_t row, uint8_t col) {
    for (uint8_t i = 0; i < pressed_key_count; i++) {
        if (pressed_keys[i][0] == row && pressed_keys[i][1] == col) {
            return true;
        }
    }
    return false;
}

// Add key to pressed tracking array
static void track_key_press(uint8_t row, uint8_t col) {
    if (!is_key_tracked(row, col) && pressed_key_count < MAX_PRESSED_KEYS) {
        pressed_keys[pressed_key_count][0] = row;
        pressed_keys[pressed_key_count][1] = col;
        pressed_key_count++;
    }
}

// Remove key from pressed tracking array
static void track_key_release(uint8_t row, uint8_t col) {
    for (uint8_t i = 0; i < pressed_key_count; i++) {
        if (pressed_keys[i][0] == row && pressed_keys[i][1] == col) {
            // Shift remaining keys down
            for (uint8_t j = i; j < pressed_key_count - 1; j++) {
                pressed_keys[j][0] = pressed_keys[j + 1][0];
                pressed_keys[j][1] = pressed_keys[j + 1][1];
            }
            pressed_key_count--;
            return;
        }
    }
}

// Add event to batch or flush and send immediately
static void add_event_to_batch(uint8_t type, uint8_t row, uint8_t col) {
    // Deduplicate: don't send press if already pressed, or release if not pressed
    if (type == MSG_KEY_PRESS) {
        if (is_key_tracked(row, col)) {
            return;  // Already pressed, skip duplicate
        }
        track_key_press(row, col);
    } else if (type == MSG_KEY_RELEASE) {
        if (!is_key_tracked(row, col)) {
            return;  // Not tracked as pressed, skip
        }
        track_key_release(row, col);
    }

    // If batch is full, flush first
    if (batch_count >= MAX_BATCH_EVENTS) {
        flush_event_batch();
    }

    // Add to batch
    event_batch[batch_count * 3] = type;
    event_batch[batch_count * 3 + 1] = row;
    event_batch[batch_count * 3 + 2] = col;
    batch_count++;
    last_batch_time = timer_read();

    // For press events, flush immediately to ensure visual guide shows it right away
    if (type == MSG_KEY_PRESS) {
        flush_event_batch();
    }
}

// Send full state to host
static void send_full_state(void) {
    uint8_t response[RAW_EPSIZE] = {0};
    response[0] = MSG_FULL_STATE;
    response[1] = get_effective_layer();
    response[2] = is_caps_word_on() ? 1 : 0;
    response[3] = get_modifier_state();
    // Note: We don't track pressed keys in firmware, so count is 0
    // The visual guide tracks this from press/release events
    response[4] = 0;  // pressed key count
    raw_hid_send(response, RAW_EPSIZE);
}

// Layer state and other broadcasts via housekeeping
void housekeeping_task_user(void) {
    // Check if batch needs flushing due to timeout
    if (batch_count > 0 && timer_elapsed(last_batch_time) > BATCH_TIMEOUT_MS) {
        flush_event_batch();
    }

    // Layer state broadcast
    uint8_t current_layer = get_effective_layer();
    if (current_layer != last_broadcast_layer) {
        last_broadcast_layer = current_layer;
        uint8_t data[RAW_EPSIZE] = {0};
        data[0] = MSG_LAYER_STATE;
        data[1] = current_layer;
        raw_hid_send(data, RAW_EPSIZE);
    }

    // Caps Word state broadcast
    bool current_caps_word = is_caps_word_on();
    if (current_caps_word != last_caps_word_state) {
        last_caps_word_state = current_caps_word;
        uint8_t data[RAW_EPSIZE] = {0};
        data[0] = MSG_CAPS_WORD_STATE;
        data[1] = current_caps_word ? 1 : 0;
        raw_hid_send(data, RAW_EPSIZE);
    }

    // Modifier state broadcast
    uint8_t current_mods = get_modifier_state();
    if (current_mods != last_modifier_state) {
        last_modifier_state = current_mods;
        uint8_t data[RAW_EPSIZE] = {0};
        data[0] = MSG_MODIFIER_STATE;
        data[1] = current_mods;
        raw_hid_send(data, RAW_EPSIZE);
    }
}

// Keypress broadcast with batching
bool process_record_user(uint16_t keycode, keyrecord_t *record) {
    uint8_t type = record->event.pressed ? MSG_KEY_PRESS : MSG_KEY_RELEASE;
    uint8_t row = record->event.key.row;
    uint8_t col = record->event.key.col;

    // Add to batch for efficient transmission
    add_event_to_batch(type, row, col);

    return true;
}

// Post-process hook - catches any events that might be delayed by tap-hold processing
void post_process_record_user(uint16_t keycode, keyrecord_t *record) {
    // The deduplication in add_event_to_batch will prevent double-sending
    // This serves as a safety net for mod-tap and other delayed key events
    uint8_t type = record->event.pressed ? MSG_KEY_PRESS : MSG_KEY_RELEASE;
    uint8_t row = record->event.key.row;
    uint8_t col = record->event.key.col;

    add_event_to_batch(type, row, col);
}

// Handle HID requests from host
bool raw_hid_receive_kb(uint8_t *data, uint8_t length) {
    switch (data[0]) {
        case MSG_REQUEST_STATE:
            // Send full state response
            send_full_state();
            return true;

        case MSG_HEARTBEAT:
            // Respond with full state as acknowledgment
            send_full_state();
            return true;

        default:
            break;
    }
    return false;
}

// RGB Matrix layer indication
#ifdef RGB_MATRIX_ENABLE

// LED index mapping (matches keyboard.json rgb_matrix layout order)
// Left hand: LEDs 0-17 (rows 0-2: 5 keys each, row 3: 3 thumbs)
// Right hand: LEDs 18-35 (rows 0-2: 5 keys each, row 3: 3 thumbs)
//
// Left hand LED layout (inner to outer wiring per row):
//   Row 0: 0(col4) 1(col3) 2(col2) 3(col1) 4(col0)
//   Row 1: 5(col4) 6(col3) 7(col2) 8(col1) 9(col0)
//   Row 2: 10(col4) 11(col3) 12(col2) 13(col1) 14(col0)
//   Thumb:  15(col4) 16(col3) 17(col2)
//
// Right hand LED layout:
//   Row 0: 18(col4) 19(col3) 20(col2) 21(col1) 22(col0)
//   Row 1: 23(col4) 24(col3) 25(col2) 26(col1) 27(col0)
//   Row 2: 28(col4) 29(col3) 30(col2) 31(col1) 32(col0)
//   Thumb:  33(col2) 34(col1) 35(col0)

const uint8_t FINGER_MAP[36] = {
    // Left hand LEDs wired inner-to-outer: inner, index, middle, ring, pinky
    3, 3, 2, 1, 0,  3, 3, 2, 1, 0,  3, 3, 2, 1, 0,  4, 4, 4,
    // Right hand LEDs wired outer-to-inner: pinky, ring, middle, index, inner
    0, 1, 2, 3, 3,  0, 1, 2, 3, 3,  0, 1, 2, 3, 3,  4, 4, 4
};

// HSV colors for finger identification (base layers)
const uint8_t FINGER_COLORS[][3] = {
    {128, 255, 180}, // 0: pinky - cyan
    {213, 255, 180}, // 1: ring - purple
    {85, 255, 180},  // 2: middle - green
    {43, 255, 180},  // 3: index/inner - yellow
    {170, 255, 180}  // 4: thumb - blue
};

// Helper to set LED color from HSV
static inline void set_led_hsv(uint8_t led, uint8_t h, uint8_t s, uint8_t v) {
    HSV hsv = {h, s, v};
    RGB rgb = hsv_to_rgb(hsv);
    rgb_matrix_set_color(led, rgb.r, rgb.g, rgb.b);
}

bool rgb_matrix_indicators_advanced_user(uint8_t led_min, uint8_t led_max) {
    uint8_t layer = get_effective_layer();

    // Base layers (0-2): Show finger colors for home row identification
    if (layer <= 2) {
        for (uint8_t i = led_min; i < led_max && i < 36; i++) {
            uint8_t finger = FINGER_MAP[i];
            set_led_hsv(i, FINGER_COLORS[finger][0], FINGER_COLORS[finger][1], FINGER_COLORS[finger][2]);
        }
        return false;
    }

    // For layers 3-9, color ALL keys in the layer's designated color
    uint8_t h, s, v;

    switch (layer) {
        case 3:  // BUTTON - orange
            h = 21; s = 255; v = 200;
            break;
        case 4:  // NAV - cyan
            h = 128; s = 255; v = 200;
            break;
        case 5:  // MOUSE - yellow
            h = 43; s = 255; v = 200;
            break;
        case 6:  // MEDIA - purple
            h = 213; s = 255; v = 200;
            break;
        case 7:  // NUM - blue
            h = 170; s = 255; v = 200;
            break;
        case 8:  // SYM - green
            h = 85; s = 255; v = 200;
            break;
        case 9:  // FUN - red
            h = 0; s = 255; v = 200;
            break;
        default:
            h = 0; s = 0; v = 200;  // white fallback
            break;
    }

    // Apply the solid color to all LEDs in range
    for (uint8_t i = led_min; i < led_max && i < 36; i++) {
        set_led_hsv(i, h, s, v);
    }

    return false;
}
#endif
