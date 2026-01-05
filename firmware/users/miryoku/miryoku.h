// Copyright 2019 Manna Harbour
// https://github.com/manna-harbour/miryoku
// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

#include "miryoku_babel/miryoku_layer_list.h"

enum miryoku_layers {
#define MIRYOKU_X(LAYER, STRING) U_##LAYER,
MIRYOKU_LAYER_LIST
#undef MIRYOKU_X
};

#if !defined (LAYOUT_miryoku)
#define LAYOUT_miryoku LAYOUT_split_3x5_3
#endif

#define U_NP KC_NO
#define U_NA KC_NO
#define U_NU KC_NO

#if defined (MIRYOKU_CLIPBOARD_FUN)
#define U_RDO KC_AGIN
#define U_PST KC_PSTE
#define U_CPY KC_COPY
#define U_CUT KC_CUT
#define U_UND KC_UNDO
#elif defined (MIRYOKU_CLIPBOARD_MAC)
#define U_RDO SCMD(KC_Z)
#define U_PST LCMD(KC_V)
#define U_CPY LCMD(KC_C)
#define U_CUT LCMD(KC_X)
#define U_UND LCMD(KC_Z)
#elif defined (MIRYOKU_CLIPBOARD_WIN)
#define U_RDO C(KC_Y)
#define U_PST C(KC_V)
#define U_CPY C(KC_C)
#define U_CUT C(KC_X)
#define U_UND C(KC_Z)
#else
#define U_RDO KC_AGIN
#define U_PST S(KC_INS)
#define U_CPY C(KC_INS)
#define U_CUT S(KC_DEL)
#define U_UND KC_UNDO
#endif
