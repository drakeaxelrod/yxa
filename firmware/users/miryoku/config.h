// Copyright 2019 Manna Harbour
// https://github.com/manna-harbour/miryoku
// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

#define TAPPING_TERM 220
#define QUICK_TAP_TERM 0

#define AUTO_SHIFT_TIMEOUT TAPPING_TERM
#define AUTO_SHIFT_NO_SETUP
#define NO_AUTO_SHIFT_ALPHA

#define MOUSEKEY_DELAY 0
#define MOUSEKEY_INTERVAL 16
#define MOUSEKEY_MAX_SPEED 6
#define MOUSEKEY_TIME_TO_MAX 64

#if defined (MIRYOKU_KLUDGE_THUMBCOMBOS)
#define COMBO_COUNT 8
#define COMBO_TERM 200
#define EXTRA_SHORT_COMBOS
#endif

#include "custom_config.h"
