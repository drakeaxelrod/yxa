// Copyright 2025 QMK
// SPDX-License-Identifier: GPL-2.0-or-later

#include QMK_KEYBOARD_H
#include "raw_hid.h"

// Layer broadcast message types
#define MSG_LAYER_STATE 0x01
#define MSG_REQUEST_STATE 0x00

#ifndef RAW_EPSIZE
#define RAW_EPSIZE 32
#endif

static uint8_t last_broadcast_layer = 255;

enum layers {
    _LAYER_0,
    _LAYER_1,
    _LAYER_2,
    _LAYER_3,
};

#define LAYER0 MO(_LAYER_0)
#define LAYER1 MO(_LAYER_1)
#define LAYER2 MO(_LAYER_2)
#define LAYER3 MO(_LAYER_3)

#define Z_SFT   LSFT_T(KC_Z)
#define SLS_SFT RSFT_T(KC_SLSH)
#define BSL_SFT RSFT_T(KC_BSLS)
#define DEL_ALT LALT_T(KC_DEL)
#define TAB_GUI LGUI_T(KC_TAB)
#define ESC_CTL LCTL_T(KC_ESC)

#define G_TAB G(KC_TAB)
#define C_ESC C(KC_ESC)

#define ENT_LY1 LT(_LAYER_1, KC_ENT)
#define BSP_LY2 LT(_LAYER_2, KC_BSPC)

const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {

  [_LAYER_0] = LAYOUT_split_3x5_3(
    KC_Q,    KC_W,    KC_E,    KC_R,    KC_T,         KC_Y,    KC_U,    KC_I,    KC_O,    KC_P,
    KC_A,    KC_S,    KC_D,    KC_F,    KC_G,         KC_H,    KC_J,    KC_K,    KC_L,    KC_SCLN,
    Z_SFT,   KC_X,    KC_C,    KC_V,    KC_B,         KC_N,    KC_M,    KC_COMM, KC_DOT,  SLS_SFT,
                      DEL_ALT, TAB_GUI, ESC_CTL,      KC_SPC,  ENT_LY1, BSP_LY2
  ),

  [_LAYER_1] = LAYOUT_split_3x5_3(
    KC_1,    KC_2,    KC_3,    KC_4,    KC_5,         KC_6,    KC_7,    KC_8,    KC_9,    KC_0,
    KC_GRV,  KC_HOME, KC_PGDN, KC_PGUP, KC_END,       KC_LEFT, KC_DOWN, KC_UP,   KC_RGHT, KC_QUOT,
    KC_LSFT, KC_BRID, KC_BRIU, XXXXXXX, XXXXXXX,      KC_MINS, KC_EQL,  KC_LBRC, KC_RBRC, BSL_SFT,
                      XXXXXXX, G_TAB,   C_ESC,        XXXXXXX, _______, LAYER3
  ),

  [_LAYER_2] = LAYOUT_split_3x5_3(
    KC_F1,   KC_F2,   KC_F3,   KC_F4,   KC_F5,        KC_F6,   KC_F7,   KC_F8,   KC_F9,   KC_F10,
    KC_F11,  KC_F12,  KC_MPRV, KC_MPLY, KC_MNXT,      MS_LEFT, MS_DOWN, MS_UP,   MS_RGHT, KC_PSCR,
    KC_MUTE, KC_VOLD, KC_VOLU, MS_WHLD, MS_WHLU,      MS_BTN1, MS_BTN2, MS_BTN3, MS_BTN4, MS_BTN5,
                      MS_ACL0, MS_ACL1, MS_ACL2,      XXXXXXX, LAYER3,  _______
  ),

  [_LAYER_3] = LAYOUT_split_3x5_3(
    QK_BOOT, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX,      XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX,
    RM_TOGG, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX,      XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX,
    RM_NEXT, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX,      XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX, XXXXXXX,
                      XXXXXXX, XXXXXXX, XXXXXXX,      XXXXXXX, _______, _______
  )

};

// Broadcast layer state when it changes
void housekeeping_task_user(void) {
    uint8_t current = get_highest_layer(layer_state);

    if (current != last_broadcast_layer) {
        last_broadcast_layer = current;

        uint8_t data[RAW_EPSIZE] = {0};
        data[0] = MSG_LAYER_STATE;
        data[1] = current;
        raw_hid_send(data, RAW_EPSIZE);
    }
}

// Handle layer state requests from overlay (uses _kb to not conflict with Vial)
bool raw_hid_receive_kb(uint8_t *data, uint8_t length) {
    if (data[0] == MSG_REQUEST_STATE) {
        uint8_t response[RAW_EPSIZE] = {0};
        response[0] = MSG_LAYER_STATE;
        response[1] = get_highest_layer(layer_state);
        raw_hid_send(response, RAW_EPSIZE);
        return true;
    }
    return false;
}

// RGB Layer Indication
#ifdef RGB_MATRIX_ENABLE

// Map key positions to fingers (for a split_3x5_3 layout)
const uint8_t FINGER_MAP[36] = {
    // Left hand row 0
    0, 1, 2, 3, 3,  // pinky, ring, middle, index, index
    // Left hand row 1
    0, 1, 2, 3, 3,
    // Left hand row 2
    0, 1, 2, 3, 3,
    // Left thumbs
    4, 4, 4,        // thumb cluster

    // Right hand row 0
    3, 3, 2, 1, 0,  // index, index, middle, ring, pinky
    // Right hand row 1
    3, 3, 2, 1, 0,
    // Right hand row 2
    3, 3, 2, 1, 0,
    // Right thumbs
    4, 4, 4
};

// HSV colors for each finger
const uint8_t FINGER_COLORS[][3] = {
    {128, 255, 180},  // 0: Pinky - Cyan
    {213, 255, 180},  // 1: Ring - Magenta
    { 85, 255, 180},  // 2: Middle - Green
    { 43, 255, 180},  // 3: Index - Yellow
    {170, 255, 180},  // 4: Thumb - Blue
};

// Layer colors (HSV)
const uint8_t LAYER_COLORS[][3] = {
    {  0,   0, 128},  // 0: BASE - White (dim)
    {128, 255, 200},  // 1: NAV - Cyan
    { 85, 255, 200},  // 2: MOUSE - Green
    {213, 255, 200},  // 3: MEDIA - Magenta
    { 43, 255, 200},  // 4: NUM - Yellow
    {  0, 255, 200},  // 5: SYM - Red
    {170, 255, 200},  // 6: FUN - Blue
    { 21, 255, 200},  // 7: BUTTON - Orange
};

bool rgb_matrix_indicators_advanced_user(uint8_t led_min, uint8_t led_max) {
    uint8_t layer = get_highest_layer(layer_state);

    for (uint8_t i = led_min; i < led_max; i++) {
        if (layer == 0) {
            // Base layer: color by finger
            if (i < 36) {
                uint8_t finger = FINGER_MAP[i];
                HSV hsv = {FINGER_COLORS[finger][0], FINGER_COLORS[finger][1], FINGER_COLORS[finger][2]};
                RGB rgb = hsv_to_rgb(hsv);
                rgb_matrix_set_color(i, rgb.r, rgb.g, rgb.b);
            }
        } else if (layer < 8) {
            // Other layers: solid color
            HSV hsv = {LAYER_COLORS[layer][0], LAYER_COLORS[layer][1], LAYER_COLORS[layer][2]};
            RGB rgb = hsv_to_rgb(hsv);
            rgb_matrix_set_color(i, rgb.r, rgb.g, rgb.b);
        }
    }
    return false;
}

#endif // RGB_MATRIX_ENABLE
