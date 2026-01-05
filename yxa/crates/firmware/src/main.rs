//! Yxa 36-Key Split Ergonomic Keyboard Firmware
//!
//! Keyberon-based firmware for the Yxa (formerly SZR35) split keyboard.
//! Uses STM32F401CCU6 with native DFU bootloader.
//!
//! Hardware notes:
//! - Direct pin matrix (each key has its own pin, no diodes)
//! - Split keyboard with half-duplex USART1 on PA9
//! - Handedness detection via PC7 pin
//! - RGB LEDs on PA7 (WS2812 via TIM3 PWM)
//! - 16MHz HSE crystal (not 25MHz like WeAct BlackPill!)
//! - PLL config: PLLM=16, PLLN=192, PLLQ=4 → 48MHz USB clock

#![no_std]
#![no_main]

use panic_halt as _;

use core::convert::Infallible;
use keyberon::debounce::Debouncer;
use keyberon::key_code::KeyCode::*;
use keyberon::layout::{Event, Layout};
use keyberon::matrix::PressedKeys;
use rtic::app;
use rtic_monotonics::systick::prelude::*;
use stm32f4xx_hal::gpio::{gpioa, gpiob, gpioc, ErasedPin, Input, PullUp};
use stm32f4xx_hal::otg_fs::{UsbBus, UsbBusType, USB};
use stm32f4xx_hal::pac::TIM2;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::timer::{CounterHz, Event as TimerEvent};
use usb_device::bus::UsbBusAllocator;
use usb_device::prelude::*;

// Re-export for layout macro
use keyberon::action::Action::*;
use keyberon::action::{d, k, l, m, Action, HoldTapAction, HoldTapConfig};

/// USB VID/PID - using pid.codes for open source
const VID: u16 = 0x1209;
const PID: u16 = 0x0036;

/// Matrix dimensions per half
const NCOLS: usize = 5;
const NROWS: usize = 4;
/// Total rows in layout (both halves)
const NROWS_TOTAL: usize = NROWS * 2;
/// Number of layers
const NLAYERS: usize = 7;

// ============================================================================
// Layout Definition
// ============================================================================

type Layers = keyberon::layout::Layers<NCOLS, NROWS_TOTAL, NLAYERS, ()>;

/// Home row mod-tap: tap for key, hold for modifier
macro_rules! hrm {
    ($tap:expr, $hold:expr) => {
        Action::HoldTap(&HoldTapAction {
            timeout: 200,
            hold: k($hold),
            tap: k($tap),
            config: HoldTapConfig::Default,
            tap_hold_interval: 0,
        })
    };
}

/// Layer-tap: tap for key, hold for layer
macro_rules! lt {
    ($tap:expr, $layer:expr) => {
        Action::HoldTap(&HoldTapAction {
            timeout: 200,
            hold: l($layer),
            tap: k($tap),
            config: HoldTapConfig::Default,
            tap_hold_interval: 0,
        })
    };
}

// Layer indices
const BASE: usize = 0;
const NAV: usize = 1;
const MOUSE: usize = 2;
const MEDIA: usize = 3;
const NUM: usize = 4;
const SYM: usize = 5;
const FUN: usize = 6;

// Home row mods (Colemak-DH)
const A_GUI: Action<()> = hrm!(A, LGui);
const R_ALT: Action<()> = hrm!(R, LAlt);
const S_CTL: Action<()> = hrm!(S, LCtrl);
const T_SFT: Action<()> = hrm!(T, LShift);
const N_SFT: Action<()> = hrm!(N, RShift);
const E_CTL: Action<()> = hrm!(E, RCtrl);
const I_ALT: Action<()> = hrm!(I, RAlt);
const O_GUI: Action<()> = hrm!(O, RGui);

// Thumb layer-taps
const ESC_MED: Action<()> = lt!(Escape, MEDIA);
const SPC_NAV: Action<()> = lt!(Space, NAV);
const TAB_MOU: Action<()> = lt!(Tab, MOUSE);
const ENT_SYM: Action<()> = lt!(Enter, SYM);
const BS_NUM: Action<()> = lt!(BSpace, NUM);
const DEL_FUN: Action<()> = lt!(Delete, FUN);

// Shorthand
const ___: Action<()> = Trans;
const XXX: Action<()> = NoOp;

#[rustfmt::skip]
static LAYERS: Layers = [
    // =========================================================================
    // BASE - Colemak-DH with home row mods
    // =========================================================================
    [
        // Left hand (rows 0-3)
        [k(Q),    k(W),    k(F),    k(P),    k(B)],
        [A_GUI,   R_ALT,   S_CTL,   T_SFT,   k(G)],
        [k(Z),    k(X),    k(C),    k(D),    k(V)],
        [XXX,     XXX,     ESC_MED, SPC_NAV, TAB_MOU],
        // Right hand (rows 4-7)
        [k(J),    k(L),    k(U),    k(Y),    k(Quote)],
        [k(M),    N_SFT,   E_CTL,   I_ALT,   O_GUI],
        [k(K),    k(H),    k(Comma),k(Dot),  k(Slash)],
        [ENT_SYM, BS_NUM,  DEL_FUN, XXX,     XXX],
    ],
    // =========================================================================
    // NAV - Navigation (activated by right thumb, left hand has mods)
    // =========================================================================
    [
        [___,     ___,     ___,     ___,     ___],
        [k(LGui), k(LAlt), k(LCtrl),k(LShift),___],
        [___,     ___,     ___,     ___,     ___],
        [XXX,     XXX,     ___,     ___,     ___],
        // Right hand - navigation
        [k(PgUp), k(Home), k(Up),   k(End),  k(Insert)],
        [k(PgDown),k(Left),k(Down), k(Right),k(CapsLock)],
        [___,     k(BSpace),k(Delete),___,   ___],
        [k(Enter),k(BSpace),k(Delete),XXX,   XXX],
    ],
    // =========================================================================
    // MOUSE - placeholder (keyberon lacks mouse support)
    // =========================================================================
    [
        [___,     ___,     ___,     ___,     ___],
        [k(LGui), k(LAlt), k(LCtrl),k(LShift),___],
        [___,     ___,     ___,     ___,     ___],
        [XXX,     XXX,     ___,     ___,     ___],
        [___,     ___,     ___,     ___,     ___],
        [___,     ___,     ___,     ___,     ___],
        [___,     ___,     ___,     ___,     ___],
        [___,     ___,     ___,     XXX,     XXX],
    ],
    // =========================================================================
    // MEDIA - Media controls
    // =========================================================================
    [
        [___,     ___,     ___,     ___,     ___],
        [k(LGui), k(LAlt), k(LCtrl),k(LShift),___],
        [___,     ___,     ___,     ___,     ___],
        [XXX,     XXX,     ___,     ___,     ___],
        // Right hand - media
        [___,     ___,     k(VolUp),___,     ___],
        [___,     k(MediaPreviousSong),k(VolDown),k(MediaNextSong),___],
        [___,     ___,     k(Mute), ___,     ___],
        [k(MediaStop),k(MediaPlayPause),___,XXX,XXX],
    ],
    // =========================================================================
    // NUM - Number pad (activated by left thumb, right hand has mods)
    // =========================================================================
    [
        // Left hand - numpad
        [k(LBracket),k(Kb7), k(Kb8), k(Kb9), k(RBracket)],
        [k(SColon), k(Kb4), k(Kb5), k(Kb6), k(Equal)],
        [k(Grave), k(Kb1), k(Kb2), k(Kb3), k(Bslash)],
        [XXX,      XXX,    k(Dot), k(Kb0), k(Minus)],
        // Right hand - mods
        [___,     ___,     ___,     ___,     ___],
        [___,     k(RShift),k(RCtrl),k(RAlt),k(RGui)],
        [___,     ___,     ___,     ___,     ___],
        [___,     ___,     ___,     XXX,     XXX],
    ],
    // =========================================================================
    // SYM - Symbols (shifted numbers)
    // =========================================================================
    [
        // Left hand - symbols
        [m(&[LShift, LBracket].as_slice()),m(&[LShift, Kb7].as_slice()),m(&[LShift, Kb8].as_slice()),m(&[LShift, Kb9].as_slice()),m(&[LShift, RBracket].as_slice())],
        [m(&[LShift, SColon].as_slice()),m(&[LShift, Kb4].as_slice()),m(&[LShift, Kb5].as_slice()),m(&[LShift, Kb6].as_slice()),m(&[LShift, Equal].as_slice())],
        [m(&[LShift, Grave].as_slice()),m(&[LShift, Kb1].as_slice()),m(&[LShift, Kb2].as_slice()),m(&[LShift, Kb3].as_slice()),m(&[LShift, Bslash].as_slice())],
        [XXX,      XXX,    m(&[LShift, Kb9].as_slice()),m(&[LShift, Kb0].as_slice()),m(&[LShift, Minus].as_slice())],
        // Right hand - mods
        [___,     ___,     ___,     ___,     ___],
        [___,     k(RShift),k(RCtrl),k(RAlt),k(RGui)],
        [___,     ___,     ___,     ___,     ___],
        [___,     ___,     ___,     XXX,     XXX],
    ],
    // =========================================================================
    // FUN - Function keys
    // =========================================================================
    [
        // Left hand - F keys
        [k(F12),  k(F7),   k(F8),   k(F9),   k(PScreen)],
        [k(F11),  k(F4),   k(F5),   k(F6),   k(ScrollLock)],
        [k(F10),  k(F1),   k(F2),   k(F3),   k(Pause)],
        [XXX,     XXX,     k(Application),k(Space),k(Tab)],
        // Right hand - mods
        [___,     ___,     ___,     ___,     ___],
        [___,     k(RShift),k(RCtrl),k(RAlt),k(RGui)],
        [___,     ___,     ___,     ___,     ___],
        [___,     ___,     ___,     XXX,     XXX],
    ],
];

// ============================================================================
// Direct Pin Matrix
// ============================================================================

/// Direct pin matrix - each key has its own GPIO pin
/// Reads all pins and returns pressed state
pub struct DirectPins {
    /// Pins organized as [row][col], None for unused positions
    pins: [[Option<ErasedPin<Input>>; NCOLS]; NROWS],
}

impl DirectPins {
    /// Read current state of all keys
    /// Returns true for each position where the key is pressed (pin low)
    pub fn get(&self) -> PressedKeys<NCOLS, NROWS> {
        let mut pressed = PressedKeys::default();
        for (row, row_pins) in self.pins.iter().enumerate() {
            for (col, pin) in row_pins.iter().enumerate() {
                if let Some(p) = pin {
                    // Active low - pressed when pin reads low
                    if p.is_low() {
                        pressed.0[row][col] = true;
                    }
                }
            }
        }
        pressed
    }
}

// ============================================================================
// USB
// ============================================================================

static mut USB_BUS: Option<UsbBusAllocator<UsbBusType>> = None;
static mut EP_MEMORY: [u32; 1024] = [0; 1024];

// ============================================================================
// RTIC Application
// ============================================================================

systick_monotonic!(Mono, 1000);

#[app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1, SPI2])]
mod app {
    use super::*;

    #[shared]
    struct Shared {
        usb_dev: UsbDevice<'static, UsbBusType>,
        usb_class: keyberon::Class<'static, UsbBusType, ()>,
    }

    #[local]
    struct Local {
        matrix: DirectPins,
        debouncer: Debouncer<PressedKeys<NCOLS, NROWS>>,
        layout: Layout<NCOLS, NROWS_TOTAL, NLAYERS, ()>,
        timer: CounterHz<TIM2>,
        is_right: bool,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let dp = cx.device;

        // Clock configuration from board.h/mcuconf.h:
        // HSE = 16MHz, PLLM=16, PLLN=192, PLLQ=4
        // This gives: VCO = 16/16 * 192 = 192MHz
        // SYSCLK = 192/2 = 96MHz (PLLP default /2)
        // USB = 192/4 = 48MHz (PLLQ=4)
        let rcc = dp.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(16.MHz())
            .sysclk(96.MHz())
            .require_pll48clk()
            .freeze();

        // Start monotonic timer with actual sysclk
        Mono::start(cx.core.SYST, 96_000_000);

        // GPIO ports
        let gpioa = dp.GPIOA.split();
        let gpiob = dp.GPIOB.split();
        let gpioc = dp.GPIOC.split();

        // Handedness detection: PC7
        // According to QMK: this pin determines handedness
        // We'll read it - typically one side has it pulled high, other low
        let hand_pin = gpioc.pc7.into_pull_up_input();
        let is_right = hand_pin.is_low();

        // Build the matrix based on which half we are
        // Pin mappings from keyboard.json
        let matrix = if is_right {
            // RIGHT half pins:
            // Row 0: B2, B10, C10, C11, B6
            // Row 1: B0, B1, C12, (D2 N/A), B7
            // Row 2: C5, B12, B3, B4, B5
            // Row 3: C4, A1, C13, NO_PIN, NO_PIN
            //
            // Note: D2 doesn't exist on STM32F401CCU6 (48-pin)
            // This might be a PCB error or the key doesn't exist
            DirectPins {
                pins: [
                    [
                        Some(gpiob.pb2.into_pull_up_input().erase()),
                        Some(gpiob.pb10.into_pull_up_input().erase()),
                        Some(gpioc.pc10.into_pull_up_input().erase()),
                        Some(gpioc.pc11.into_pull_up_input().erase()),
                        Some(gpiob.pb6.into_pull_up_input().erase()),
                    ],
                    [
                        Some(gpiob.pb0.into_pull_up_input().erase()),
                        Some(gpiob.pb1.into_pull_up_input().erase()),
                        Some(gpioc.pc12.into_pull_up_input().erase()),
                        None, // D2 not available - key may not exist
                        Some(gpiob.pb7.into_pull_up_input().erase()),
                    ],
                    [
                        Some(gpioc.pc5.into_pull_up_input().erase()),
                        Some(gpiob.pb12.into_pull_up_input().erase()),
                        Some(gpiob.pb3.into_pull_up_input().erase()),
                        Some(gpiob.pb4.into_pull_up_input().erase()),
                        Some(gpiob.pb5.into_pull_up_input().erase()),
                    ],
                    [
                        Some(gpioc.pc4.into_pull_up_input().erase()),
                        Some(gpioa.pa1.into_pull_up_input().erase()),
                        Some(gpioc.pc13.into_pull_up_input().erase()),
                        None, // NO_PIN
                        None, // NO_PIN
                    ],
                ],
            }
        } else {
            // LEFT half pins:
            // Row 0: B4, B0, B1, B12, B13
            // Row 1: B5, C5, B2, B14, B15
            // Row 2: B3, C4, B10, C9, C11
            // Row 3: NO_PIN, NO_PIN, A1, C14, C12
            DirectPins {
                pins: [
                    [
                        Some(gpiob.pb4.into_pull_up_input().erase()),
                        Some(gpiob.pb0.into_pull_up_input().erase()),
                        Some(gpiob.pb1.into_pull_up_input().erase()),
                        Some(gpiob.pb12.into_pull_up_input().erase()),
                        Some(gpiob.pb13.into_pull_up_input().erase()),
                    ],
                    [
                        Some(gpiob.pb5.into_pull_up_input().erase()),
                        Some(gpioc.pc5.into_pull_up_input().erase()),
                        Some(gpiob.pb2.into_pull_up_input().erase()),
                        Some(gpiob.pb14.into_pull_up_input().erase()),
                        Some(gpiob.pb15.into_pull_up_input().erase()),
                    ],
                    [
                        Some(gpiob.pb3.into_pull_up_input().erase()),
                        Some(gpioc.pc4.into_pull_up_input().erase()),
                        Some(gpiob.pb10.into_pull_up_input().erase()),
                        Some(gpioc.pc9.into_pull_up_input().erase()),
                        Some(gpioc.pc11.into_pull_up_input().erase()),
                    ],
                    [
                        None, // NO_PIN
                        None, // NO_PIN
                        Some(gpioa.pa1.into_pull_up_input().erase()),
                        Some(gpioc.pc14.into_pull_up_input().erase()),
                        Some(gpioc.pc12.into_pull_up_input().erase()),
                    ],
                ],
            }
        };

        // USB setup
        let usb = USB::new(
            (dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK),
            (gpioa.pa11, gpioa.pa12),
            &clocks,
        );

        let usb_bus = unsafe {
            USB_BUS = Some(UsbBus::new(usb, &mut EP_MEMORY));
            USB_BUS.as_ref().unwrap()
        };

        let usb_class = keyberon::new_class(usb_bus, ());

        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(VID, PID))
            .manufacturer("Yxa")
            .product("Yxa 36")
            .serial_number(if is_right { "R" } else { "L" })
            .device_class(0x03) // HID
            .build();

        // Timer for 1kHz matrix scanning
        let mut timer = dp.TIM2.counter_hz(&clocks);
        timer.start(1.kHz()).unwrap();
        timer.listen(TimerEvent::Update);

        // Debouncer with 5ms debounce time
        let debouncer = Debouncer::new(PressedKeys::default(), PressedKeys::default(), 5);

        // Layout engine
        let layout = Layout::new(&LAYERS);

        (
            Shared { usb_dev, usb_class },
            Local {
                matrix,
                debouncer,
                layout,
                timer,
                is_right,
            },
        )
    }

    /// USB interrupt - highest priority
    #[task(binds = OTG_FS, priority = 2, shared = [usb_dev, usb_class])]
    fn usb_tx(cx: usb_tx::Context) {
        (cx.shared.usb_dev, cx.shared.usb_class).lock(|usb_dev, usb_class| {
            if usb_dev.poll(&mut [usb_class]) {
                usb_class.poll();
            }
        });
    }

    /// Matrix scan timer - 1kHz
    #[task(binds = TIM2, priority = 1, local = [matrix, debouncer, layout, timer, is_right], shared = [usb_dev, usb_class])]
    fn tick(cx: tick::Context) {
        cx.local.timer.clear_flags(TimerEvent::Update);

        // Read matrix state
        let keys = cx.local.matrix.get();

        // Debounce and extract events
        for event in cx.local.debouncer.events(keys) {
            // Transform coordinates based on which half
            // Left half: rows 0-3, Right half: rows 4-7
            let transformed = if *cx.local.is_right {
                match event {
                    Event::Press(r, c) => Event::Press(r + NROWS as u8, c),
                    Event::Release(r, c) => Event::Release(r + NROWS as u8, c),
                }
            } else {
                event
            };
            cx.local.layout.event(transformed);
        }

        // Process layout timing (for hold-tap)
        cx.local.layout.tick();

        // Generate HID report
        let report: keyberon::key_code::KbHidReport = cx.local.layout.keycodes().collect();

        // Send report
        (cx.shared.usb_dev, cx.shared.usb_class).lock(|_usb_dev, usb_class| {
            if usb_class.device_mut().set_keyboard_report(report.clone()) {
                while let Ok(0) = usb_class.write(report.as_bytes()) {}
            }
        });
    }
}
