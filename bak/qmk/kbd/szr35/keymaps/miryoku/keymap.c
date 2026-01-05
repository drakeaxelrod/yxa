// SZR35 Miryoku Keymap with Layer Broadcast + RGB Layer Indication
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

// Miryoku layers
enum layers {
    U_BASE,   // 0 - Base (Colemak-DH)
    U_NAV,    // 1 - Navigation
    U_MOUSE,  // 2 - Mouse
    U_MEDIA,  // 3 - Media
    U_NUM,    // 4 - Numbers
    U_SYM,    // 5 - Symbols
    U_FUN,    // 6 - Function keys
    U_BUTTON, // 7 - Button layer
};

// Miryoku Colemak-DH base layer with home row mods
const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {

    // BASE - Colemak-DH with home row mods
    [U_BASE] = LAYOUT_split_3x5_3(
        KC_Q,              KC_W,              KC_F,              KC_P,              KC_B,                  KC_J,              KC_L,              KC_U,              KC_Y,              KC_QUOT,
        LGUI_T(KC_A),      LALT_T(KC_R),      LCTL_T(KC_S),      LSFT_T(KC_T),      KC_G,                  KC_M,              LSFT_T(KC_N),      LCTL_T(KC_E),      LALT_T(KC_I),      LGUI_T(KC_O),
        LT(U_BUTTON,KC_Z), RALT_T(KC_X),      KC_C,              KC_D,              KC_V,                  KC_K,              KC_H,              KC_COMM,           RALT_T(KC_DOT),    LT(U_BUTTON,KC_SLSH),
                                              LT(U_MEDIA,KC_ESC),LT(U_NAV,KC_SPC),  LT(U_MOUSE,KC_TAB),    LT(U_SYM,KC_ENT),  LT(U_NUM,KC_BSPC), LT(U_FUN,KC_DEL)
    ),

    // NAV - Navigation (right hand active)
    [U_NAV] = LAYOUT_split_3x5_3(
        QK_BOOT,           XXXXXXX,           XXXXXXX,           XXXXXXX,           XXXXXXX,               KC_AGIN,           KC_PSTE,           KC_COPY,           KC_CUT,            KC_UNDO,
        KC_LGUI,           KC_LALT,           KC_LCTL,           KC_LSFT,           XXXXXXX,               CW_TOGG,           KC_LEFT,           KC_DOWN,           KC_UP,             KC_RGHT,
        XXXXXXX,           KC_RALT,           XXXXXXX,           XXXXXXX,           XXXXXXX,               KC_INS,            KC_HOME,           KC_PGDN,           KC_PGUP,           KC_END,
                                              XXXXXXX,           XXXXXXX,           XXXXXXX,               KC_ENT,            KC_BSPC,           KC_DEL
    ),

    // MOUSE - Mouse keys (right hand active)
    [U_MOUSE] = LAYOUT_split_3x5_3(
        QK_BOOT,           XXXXXXX,           XXXXXXX,           XXXXXXX,           XXXXXXX,               KC_AGIN,           KC_PSTE,           KC_COPY,           KC_CUT,            KC_UNDO,
        KC_LGUI,           KC_LALT,           KC_LCTL,           KC_LSFT,           XXXXXXX,               XXXXXXX,           KC_MS_L,           KC_MS_D,           KC_MS_U,           KC_MS_R,
        XXXXXXX,           KC_RALT,           XXXXXXX,           XXXXXXX,           XXXXXXX,               XXXXXXX,           KC_WH_L,           KC_WH_D,           KC_WH_U,           KC_WH_R,
                                              XXXXXXX,           XXXXXXX,           XXXXXXX,               KC_BTN2,           KC_BTN1,           KC_BTN3
    ),

    // MEDIA - Media controls (right hand active)
    [U_MEDIA] = LAYOUT_split_3x5_3(
        QK_BOOT,           XXXXXXX,           XXXXXXX,           XXXXXXX,           XXXXXXX,               RGB_TOG,           RGB_MOD,           RGB_HUI,           RGB_SAI,           RGB_VAI,
        KC_LGUI,           KC_LALT,           KC_LCTL,           KC_LSFT,           XXXXXXX,               XXXXXXX,           KC_MPRV,           KC_VOLD,           KC_VOLU,           KC_MNXT,
        XXXXXXX,           KC_RALT,           XXXXXXX,           XXXXXXX,           XXXXXXX,               OU_AUTO,           XXXXXXX,           XXXXXXX,           XXXXXXX,           XXXXXXX,
                                              XXXXXXX,           XXXXXXX,           XXXXXXX,               KC_MSTP,           KC_MPLY,           KC_MUTE
    ),

    // NUM - Number pad (left hand active)
    [U_NUM] = LAYOUT_split_3x5_3(
        KC_LBRC,           KC_7,              KC_8,              KC_9,              KC_RBRC,               XXXXXXX,           XXXXXXX,           XXXXXXX,           XXXXXXX,           QK_BOOT,
        KC_SCLN,           KC_4,              KC_5,              KC_6,              KC_EQL,                XXXXXXX,           KC_LSFT,           KC_LCTL,           KC_LALT,           KC_LGUI,
        KC_GRV,            KC_1,              KC_2,              KC_3,              KC_BSLS,               XXXXXXX,           XXXXXXX,           XXXXXXX,           KC_RALT,           XXXXXXX,
                                              KC_DOT,            KC_0,              KC_MINS,               XXXXXXX,           XXXXXXX,           XXXXXXX
    ),

    // SYM - Symbols (left hand active)
    [U_SYM] = LAYOUT_split_3x5_3(
        KC_LCBR,           KC_AMPR,           KC_ASTR,           KC_LPRN,           KC_RCBR,               XXXXXXX,           XXXXXXX,           XXXXXXX,           XXXXXXX,           QK_BOOT,
        KC_COLN,           KC_DLR,            KC_PERC,           KC_CIRC,           KC_PLUS,               XXXXXXX,           KC_LSFT,           KC_LCTL,           KC_LALT,           KC_LGUI,
        KC_TILD,           KC_EXLM,           KC_AT,             KC_HASH,           KC_PIPE,               XXXXXXX,           XXXXXXX,           XXXXXXX,           KC_RALT,           XXXXXXX,
                                              KC_LPRN,           KC_RPRN,           KC_UNDS,               XXXXXXX,           XXXXXXX,           XXXXXXX
    ),

    // FUN - Function keys (left hand active)
    [U_FUN] = LAYOUT_split_3x5_3(
        KC_F12,            KC_F7,             KC_F8,             KC_F9,             KC_PSCR,               XXXXXXX,           XXXXXXX,           XXXXXXX,           XXXXXXX,           QK_BOOT,
        KC_F11,            KC_F4,             KC_F5,             KC_F6,             KC_SCRL,               XXXXXXX,           KC_LSFT,           KC_LCTL,           KC_LALT,           KC_LGUI,
        KC_F10,            KC_F1,             KC_F2,             KC_F3,             KC_PAUS,               XXXXXXX,           XXXXXXX,           XXXXXXX,           KC_RALT,           XXXXXXX,
                                              KC_APP,            KC_SPC,            KC_TAB,                XXXXXXX,           XXXXXXX,           XXXXXXX
    ),

    // BUTTON - Accessible from both hands (Z and /)
    [U_BUTTON] = LAYOUT_split_3x5_3(
        KC_UNDO,           KC_CUT,            KC_COPY,           KC_PSTE,           KC_AGIN,               KC_AGIN,           KC_PSTE,           KC_COPY,           KC_CUT,            KC_UNDO,
        KC_LGUI,           KC_LALT,           KC_LCTL,           KC_LSFT,           XXXXXXX,               XXXXXXX,           KC_LSFT,           KC_LCTL,           KC_LALT,           KC_LGUI,
        KC_UNDO,           KC_CUT,            KC_COPY,           KC_PSTE,           KC_AGIN,               KC_AGIN,           KC_PSTE,           KC_COPY,           KC_CUT,            KC_UNDO,
                                              KC_BTN3,           KC_BTN1,           KC_BTN2,               KC_BTN2,           KC_BTN1,           KC_BTN3
    ),

    // Layer 8 - Transparent (for Vial compatibility, 9 layers configured)
    [8] = LAYOUT_split_3x5_3(
        KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,               KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,
        KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,               KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,
        KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,               KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,           KC_TRNS,
                                              KC_TRNS,           KC_TRNS,           KC_TRNS,               KC_TRNS,           KC_TRNS,           KC_TRNS
    ),
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
