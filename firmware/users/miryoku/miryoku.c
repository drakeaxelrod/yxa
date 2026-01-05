// Copyright 2019 Manna Harbour
// https://github.com/manna-harbour/miryoku
// SPDX-License-Identifier: GPL-2.0-or-later

#include QMK_KEYBOARD_H
#include "miryoku.h"
#include "miryoku_babel/miryoku_layer_selection.h"

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

const key_override_t capsword_override = ko_make_basic(MOD_MASK_SHIFT, CW_TOGG, KC_CAPS);
const key_override_t *key_overrides[] = {
    &capsword_override,
};

const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
#define MIRYOKU_X(LAYER, STRING) [U_##LAYER] = LAYOUT_miryoku(MIRYOKU_LAYER_##LAYER),
MIRYOKU_LAYER_LIST
#undef MIRYOKU_X
};

#if defined (MIRYOKU_KLUDGE_THUMBCOMBOS)
const uint16_t PROGMEM u_combo_esc[] = {LT(U_NAV, KC_SPC), LT(U_MOUSE, KC_TAB), COMBO_END};
const uint16_t PROGMEM u_combo_del[] = {LT(U_SYM, KC_ENT), LT(U_NUM, KC_BSPC), COMBO_END};
combo_t key_combos[] = {
    COMBO(u_combo_esc, KC_ESC),
    COMBO(u_combo_del, KC_DEL),
};
#endif
