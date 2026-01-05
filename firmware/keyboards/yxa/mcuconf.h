// Copyright 2025 QMK
// SPDX-License-Identifier: GPL-2.0-or-later

#pragma once

#include_next <mcuconf.h>

// Split keyboard serial
#undef STM32_SERIAL_USE_USART1
#define STM32_SERIAL_USE_USART1 TRUE

// WS2812 PWM on TIM3
#undef STM32_PWM_USE_TIM3
#define STM32_PWM_USE_TIM3 TRUE

// PLL configuration for 16MHz HSE (Yxa custom crystal)
// VCO = 16MHz / 16 * 192 = 192MHz
// SYSCLK = 192MHz / 2 = 96MHz (APB dividers bring peripherals to spec)
// USB = 192MHz / 4 = 48MHz (required for USB)
#undef STM32_PLLM_VALUE
#define STM32_PLLM_VALUE 16

#undef STM32_PLLN_VALUE
#define STM32_PLLN_VALUE 192

#undef STM32_PLLQ_VALUE
#define STM32_PLLQ_VALUE 4

#undef STM32_PPRE1
#define STM32_PPRE1 STM32_PPRE1_DIV4

#undef STM32_PPRE2
#define STM32_PPRE2 STM32_PPRE2_DIV2
