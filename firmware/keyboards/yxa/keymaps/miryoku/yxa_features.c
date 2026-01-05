// Yxa Keyboard - Custom Features
// SPDX-License-Identifier: GPL-2.0-or-later

#include QMK_KEYBOARD_H
#include "raw_hid.h"

#define MSG_REQUEST_STATE   0x00
#define MSG_LAYER_STATE     0x01
#define MSG_KEY_PRESS       0x02
#define MSG_KEY_RELEASE     0x03

#ifndef RAW_EPSIZE
#define RAW_EPSIZE 32
#endif

static uint8_t last_broadcast_layer = 255;

// Get effective layer (combines default layer with momentary layers)
static uint8_t get_effective_layer(void) {
    // layer_state holds momentary layers, default_layer_state holds the base layer
    // When no momentary layer is active, we want to show the default layer
    layer_state_t effective = layer_state | default_layer_state;
    return get_highest_layer(effective);
}

// Layer state broadcast via housekeeping
void housekeeping_task_user(void) {
    uint8_t current = get_effective_layer();
    if (current != last_broadcast_layer) {
        last_broadcast_layer = current;
        uint8_t data[RAW_EPSIZE] = {0};
        data[0] = MSG_LAYER_STATE;
        data[1] = current;
        raw_hid_send(data, RAW_EPSIZE);
    }
}

// Keypress broadcast
bool process_record_user(uint16_t keycode, keyrecord_t *record) {
    uint8_t data[RAW_EPSIZE] = {0};
    data[0] = record->event.pressed ? MSG_KEY_PRESS : MSG_KEY_RELEASE;
    data[1] = record->event.key.row;
    data[2] = record->event.key.col;
    raw_hid_send(data, RAW_EPSIZE);
    return true;
}

// Handle layer state requests (uses _kb to not conflict with Vial)
bool raw_hid_receive_kb(uint8_t *data, uint8_t length) {
    if (data[0] == MSG_REQUEST_STATE) {
        uint8_t response[RAW_EPSIZE] = {0};
        response[0] = MSG_LAYER_STATE;
        response[1] = get_effective_layer();
        raw_hid_send(response, RAW_EPSIZE);
        return true;
    }
    return false;
}

// RGB Matrix layer indication
#ifdef RGB_MATRIX_ENABLE
const uint8_t FINGER_MAP[36] = {
    0, 1, 2, 3, 3,  0, 1, 2, 3, 3,  0, 1, 2, 3, 3,  4, 4, 4,
    3, 3, 2, 1, 0,  3, 3, 2, 1, 0,  3, 3, 2, 1, 0,  4, 4, 4
};

const uint8_t FINGER_COLORS[][3] = {
    {128, 255, 180}, {213, 255, 180}, {85, 255, 180}, {43, 255, 180}, {170, 255, 180}
};

const uint8_t LAYER_COLORS[][3] = {
    {0, 0, 128}, {0, 0, 128}, {0, 0, 128}, {21, 255, 200},
    {128, 255, 200}, {85, 255, 200}, {213, 255, 200}, {43, 255, 200},
    {0, 255, 200}, {170, 255, 200}
};

bool rgb_matrix_indicators_advanced_user(uint8_t led_min, uint8_t led_max) {
    uint8_t layer = get_effective_layer();
    for (uint8_t i = led_min; i < led_max; i++) {
        if (layer <= 2 && i < 36) {
            uint8_t finger = FINGER_MAP[i];
            HSV hsv = {FINGER_COLORS[finger][0], FINGER_COLORS[finger][1], FINGER_COLORS[finger][2]};
            RGB rgb = hsv_to_rgb(hsv);
            rgb_matrix_set_color(i, rgb.r, rgb.g, rgb.b);
        } else if (layer < 10) {
            HSV hsv = {LAYER_COLORS[layer][0], LAYER_COLORS[layer][1], LAYER_COLORS[layer][2]};
            RGB rgb = hsv_to_rgb(hsv);
            rgb_matrix_set_color(i, rgb.r, rgb.g, rgb.b);
        }
    }
    return false;
}
#endif
