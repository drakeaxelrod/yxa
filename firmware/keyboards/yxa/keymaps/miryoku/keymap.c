// Miryoku layout for Yxa keyboard
// Based on https://github.com/manna-harbour/miryoku
// SPDX-License-Identifier: GPL-2.0-or-later

#include QMK_KEYBOARD_H

// Layer enum
enum miryoku_layers {
    U_BASE,
    U_EXTRA,
    U_TAP,
    U_BUTTON,
    U_NAV,
    U_MOUSE,
    U_MEDIA,
    U_NUM,
    U_SYM,
    U_FUN,
};

// Placeholders
#define U_NP KC_NO
#define U_NA KC_NO
#define U_NU KC_NO

// Clipboard (X11/Linux default)
#define U_RDO KC_AGIN
#define U_PST S(KC_INS)
#define U_CPY C(KC_INS)
#define U_CUT S(KC_DEL)
#define U_UND KC_UNDO

// Tap dance enum
enum {
    U_TD_BOOT,
    U_TD_U_BASE,
    U_TD_U_EXTRA,
    U_TD_U_TAP,
    U_TD_U_BUTTON,
    U_TD_U_NAV,
    U_TD_U_MOUSE,
    U_TD_U_MEDIA,
    U_TD_U_NUM,
    U_TD_U_SYM,
    U_TD_U_FUN,
};

// Tap dance functions for layer switching
void u_td_fn_boot(tap_dance_state_t *state, void *user_data) {
    if (state->count == 2) {
        reset_keyboard();
    }
}

#define U_TD_FN_LAYER(LAYER) \
void u_td_fn_##LAYER(tap_dance_state_t *state, void *user_data) { \
    if (state->count == 2) { \
        default_layer_set((layer_state_t)1 << LAYER); \
    } \
}

U_TD_FN_LAYER(U_BASE)
U_TD_FN_LAYER(U_EXTRA)
U_TD_FN_LAYER(U_TAP)

// Tap dance actions array
tap_dance_action_t tap_dance_actions[] = {
    [U_TD_BOOT] = ACTION_TAP_DANCE_FN(u_td_fn_boot),
    [U_TD_U_BASE] = ACTION_TAP_DANCE_FN(u_td_fn_U_BASE),
    [U_TD_U_EXTRA] = ACTION_TAP_DANCE_FN(u_td_fn_U_EXTRA),
    [U_TD_U_TAP] = ACTION_TAP_DANCE_FN(u_td_fn_U_TAP),
    [U_TD_U_BUTTON] = ACTION_TAP_DANCE_FN(u_td_fn_U_BASE),
    [U_TD_U_NAV] = ACTION_TAP_DANCE_FN(u_td_fn_U_BASE),
    [U_TD_U_MOUSE] = ACTION_TAP_DANCE_FN(u_td_fn_U_BASE),
    [U_TD_U_MEDIA] = ACTION_TAP_DANCE_FN(u_td_fn_U_BASE),
    [U_TD_U_NUM] = ACTION_TAP_DANCE_FN(u_td_fn_U_BASE),
    [U_TD_U_SYM] = ACTION_TAP_DANCE_FN(u_td_fn_U_BASE),
    [U_TD_U_FUN] = ACTION_TAP_DANCE_FN(u_td_fn_U_BASE),
};

// Key override: Shift + Caps Word = Caps Lock
const key_override_t capsword_override = ko_make_basic(MOD_MASK_SHIFT, CW_TOGG, KC_CAPS);
const key_override_t *key_overrides[] = {
    &capsword_override,
};

// Keymaps
const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
    // BASE - Colemak-DH with home row mods (GACS)
    [U_BASE] = LAYOUT_split_3x5_3(
        KC_Q,              KC_W,              KC_F,              KC_P,              KC_B,              KC_J,              KC_L,              KC_U,              KC_Y,              KC_QUOT,
        LGUI_T(KC_A),      LALT_T(KC_R),      LCTL_T(KC_S),      LSFT_T(KC_T),      KC_G,              KC_M,              LSFT_T(KC_N),      LCTL_T(KC_E),      LALT_T(KC_I),      LGUI_T(KC_O),
        LT(U_BUTTON,KC_Z), ALGR_T(KC_X),      KC_C,              KC_D,              KC_V,              KC_K,              KC_H,              KC_COMM,           ALGR_T(KC_DOT),    LT(U_BUTTON,KC_SLSH),
                                              LT(U_MEDIA,KC_ESC),LT(U_NAV,KC_SPC),  LT(U_MOUSE,KC_TAB),LT(U_SYM,KC_ENT),  LT(U_NUM,KC_BSPC), LT(U_FUN,KC_DEL)
    ),

    // EXTRA - QWERTY
    [U_EXTRA] = LAYOUT_split_3x5_3(
        KC_Q,              KC_W,              KC_E,              KC_R,              KC_T,              KC_Y,              KC_U,              KC_I,              KC_O,              KC_P,
        LGUI_T(KC_A),      LALT_T(KC_S),      LCTL_T(KC_D),      LSFT_T(KC_F),      KC_G,              KC_H,              LSFT_T(KC_J),      LCTL_T(KC_K),      LALT_T(KC_L),      LGUI_T(KC_QUOT),
        LT(U_BUTTON,KC_Z), ALGR_T(KC_X),      KC_C,              KC_V,              KC_B,              KC_N,              KC_M,              KC_COMM,           ALGR_T(KC_DOT),    LT(U_BUTTON,KC_SLSH),
                                              LT(U_MEDIA,KC_ESC),LT(U_NAV,KC_SPC),  LT(U_MOUSE,KC_TAB),LT(U_SYM,KC_ENT),  LT(U_NUM,KC_BSPC), LT(U_FUN,KC_DEL)
    ),

    // TAP - Colemak-DH without home row mods (but keeps layer-tap thumbs)
    [U_TAP] = LAYOUT_split_3x5_3(
        KC_Q,              KC_W,              KC_F,              KC_P,              KC_B,              KC_J,              KC_L,              KC_U,              KC_Y,              KC_QUOT,
        KC_A,              KC_R,              KC_S,              KC_T,              KC_G,              KC_M,              KC_N,              KC_E,              KC_I,              KC_O,
        KC_Z,              KC_X,              KC_C,              KC_D,              KC_V,              KC_K,              KC_H,              KC_COMM,           KC_DOT,            KC_SLSH,
                                              LT(U_MEDIA,KC_ESC),LT(U_NAV,KC_SPC),  LT(U_MOUSE,KC_TAB),LT(U_SYM,KC_ENT),  LT(U_NUM,KC_BSPC), LT(U_FUN,KC_DEL)
    ),

    // BUTTON - Clipboard and mouse buttons
    [U_BUTTON] = LAYOUT_split_3x5_3(
        U_UND,             U_CUT,             U_CPY,             U_PST,             U_RDO,             U_RDO,             U_PST,             U_CPY,             U_CUT,             U_UND,
        KC_LGUI,           KC_LALT,           KC_LCTL,           KC_LSFT,           U_NU,              U_NU,              KC_LSFT,           KC_LCTL,           KC_LALT,           KC_LGUI,
        U_UND,             U_CUT,             U_CPY,             U_PST,             U_RDO,             U_RDO,             U_PST,             U_CPY,             U_CUT,             U_UND,
                                              KC_BTN3,           KC_BTN1,           KC_BTN2,           KC_BTN2,           KC_BTN1,           KC_BTN3
    ),

    // NAV - Navigation
    [U_NAV] = LAYOUT_split_3x5_3(
        TD(U_TD_BOOT),     TD(U_TD_U_TAP),    TD(U_TD_U_EXTRA),  TD(U_TD_U_BASE),   U_NA,              U_RDO,             U_PST,             U_CPY,             U_CUT,             U_UND,
        KC_LGUI,           KC_LALT,           KC_LCTL,           KC_LSFT,           U_NA,              CW_TOGG,           KC_LEFT,           KC_DOWN,           KC_UP,             KC_RGHT,
        U_NA,              KC_ALGR,           TD(U_TD_U_NUM),    TD(U_TD_U_NAV),    U_NA,              KC_INS,            KC_HOME,           KC_PGDN,           KC_PGUP,           KC_END,
                                              U_NA,              U_NA,              U_NA,              KC_ENT,            KC_BSPC,           KC_DEL
    ),

    // MOUSE - Mouse keys
    [U_MOUSE] = LAYOUT_split_3x5_3(
        TD(U_TD_BOOT),     TD(U_TD_U_TAP),    TD(U_TD_U_EXTRA),  TD(U_TD_U_BASE),   U_NA,              U_RDO,             U_PST,             U_CPY,             U_CUT,             U_UND,
        KC_LGUI,           KC_LALT,           KC_LCTL,           KC_LSFT,           U_NA,              U_NU,              KC_MS_L,           KC_MS_D,           KC_MS_U,           KC_MS_R,
        U_NA,              KC_ALGR,           TD(U_TD_U_SYM),    TD(U_TD_U_MOUSE),  U_NA,              U_NU,              KC_WH_L,           KC_WH_D,           KC_WH_U,           KC_WH_R,
                                              U_NA,              U_NA,              U_NA,              KC_BTN2,           KC_BTN1,           KC_BTN3
    ),

    // MEDIA - Media controls and RGB
    [U_MEDIA] = LAYOUT_split_3x5_3(
        TD(U_TD_BOOT),     TD(U_TD_U_TAP),    TD(U_TD_U_EXTRA),  TD(U_TD_U_BASE),   U_NA,              RGB_TOG,           RGB_MOD,           RGB_HUI,           RGB_SAI,           RGB_VAI,
        KC_LGUI,           KC_LALT,           KC_LCTL,           KC_LSFT,           U_NA,              U_NU,              KC_MPRV,           KC_VOLD,           KC_VOLU,           KC_MNXT,
        U_NA,              KC_ALGR,           TD(U_TD_U_FUN),    TD(U_TD_U_MEDIA),  U_NA,              OU_AUTO,           U_NU,              U_NU,              U_NU,              U_NU,
                                              U_NA,              U_NA,              U_NA,              KC_MSTP,           KC_MPLY,           KC_MUTE
    ),

    // NUM - Number pad
    [U_NUM] = LAYOUT_split_3x5_3(
        KC_LBRC,           KC_7,              KC_8,              KC_9,              KC_RBRC,           U_NA,              TD(U_TD_U_BASE),   TD(U_TD_U_EXTRA),  TD(U_TD_U_TAP),    TD(U_TD_BOOT),
        KC_SCLN,           KC_4,              KC_5,              KC_6,              KC_EQL,            U_NA,              KC_LSFT,           KC_LCTL,           KC_LALT,           KC_LGUI,
        KC_GRV,            KC_1,              KC_2,              KC_3,              KC_BSLS,           U_NA,              TD(U_TD_U_NUM),    TD(U_TD_U_NAV),    KC_ALGR,           U_NA,
                                              KC_DOT,            KC_0,              KC_MINS,           U_NA,              U_NA,              U_NA
    ),

    // SYM - Symbols
    [U_SYM] = LAYOUT_split_3x5_3(
        KC_LCBR,           KC_AMPR,           KC_ASTR,           KC_LPRN,           KC_RCBR,           U_NA,              TD(U_TD_U_BASE),   TD(U_TD_U_EXTRA),  TD(U_TD_U_TAP),    TD(U_TD_BOOT),
        KC_COLN,           KC_DLR,            KC_PERC,           KC_CIRC,           KC_PLUS,           U_NA,              KC_LSFT,           KC_LCTL,           KC_LALT,           KC_LGUI,
        KC_TILD,           KC_EXLM,           KC_AT,             KC_HASH,           KC_PIPE,           U_NA,              TD(U_TD_U_SYM),    TD(U_TD_U_MOUSE),  KC_ALGR,           U_NA,
                                              KC_LPRN,           KC_RPRN,           KC_UNDS,           U_NA,              U_NA,              U_NA
    ),

    // FUN - Function keys
    [U_FUN] = LAYOUT_split_3x5_3(
        KC_F12,            KC_F7,             KC_F8,             KC_F9,             KC_PSCR,           U_NA,              TD(U_TD_U_BASE),   TD(U_TD_U_EXTRA),  TD(U_TD_U_TAP),    TD(U_TD_BOOT),
        KC_F11,            KC_F4,             KC_F5,             KC_F6,             KC_SCRL,           U_NA,              KC_LSFT,           KC_LCTL,           KC_LALT,           KC_LGUI,
        KC_F10,            KC_F1,             KC_F2,             KC_F3,             KC_PAUS,           U_NA,              TD(U_TD_U_FUN),    TD(U_TD_U_MEDIA),  KC_ALGR,           U_NA,
                                              KC_APP,            KC_SPC,            KC_TAB,            U_NA,              U_NA,              U_NA
    ),
};
