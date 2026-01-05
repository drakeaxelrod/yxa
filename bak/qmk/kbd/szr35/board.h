// Copyright 2025 QMK
// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

#include_next <board.h>

// SZR35 uses 16MHz external crystal instead of BLACKPILL's 25MHz
#undef STM32_HSECLK
#define STM32_HSECLK 16000000U
