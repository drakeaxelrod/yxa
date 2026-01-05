// Copyright 2025 Yxa
// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

#define USB_POLLING_INTERVAL_MS 1
#define DEBOUNCE 5

// WS2812 PWM driver configuration
#define WS2812_PWM_DRIVER PWMD3
#define WS2812_DMA_STREAM STM32_DMA1_STREAM2
#define WS2812_DMA_CHANNEL 5

// Mouse key settings - constant speed, no acceleration
#define MOUSEKEY_DELAY 0
#define MOUSEKEY_INTERVAL 16
#define MOUSEKEY_MOVE_DELTA 3
#define MOUSEKEY_MAX_SPEED 3
#define MOUSEKEY_TIME_TO_MAX 0
// Scroll wheel
#define MOUSEKEY_WHEEL_DELAY 0
#define MOUSEKEY_WHEEL_INTERVAL 80
#define MOUSEKEY_WHEEL_DELTA 1
#define MOUSEKEY_WHEEL_MAX_SPEED 1
#define MOUSEKEY_WHEEL_TIME_TO_MAX 0

// Raw HID for visual guide communication
#define RAW_USAGE_PAGE 0xFF60
#define RAW_USAGE_ID 0x61
#define YXA_HID_KEYPRESS_ENABLED 1
