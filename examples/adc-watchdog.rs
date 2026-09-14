//! ADC analog watchdog: an alarm when PA4 leaves the lower half of the range.
//!
//! `main` converts PA4 twice a second, forever; the window is `0..=2047`, so a
//! read above mid-range trips the watchdog and `ADC_CMP` clears the flag and
//! counts the trip. Move PA4 between GND and +3V3 by hand: a low level reports
//! in range, a high one warns. Results go over RTT.
//!
//! Covers: `set_watchdog_window`, `enable_watchdog`, and `listen`,
//! `is_pending` and `clear_interrupt` for `Event::Watchdog`.

#![no_std]
#![no_main]

use core::cell::RefCell;
use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m::peripheral::NVIC;
use cortex_m_rt::entry;
use critical_section::Mutex;
use defmt_rtt as _;
use panic_halt as _;

use gd32e2_hal::adc::{Adc, Event, SampTime};
use gd32e2_hal::pac::{self, interrupt};
use gd32e2_hal::prelude::*;
use gd32e2_hal::rcu::{AdcPsc, AdcSel, ClockConfig, PllFreq, SysClk};

/// The window: the lower half of the 12-bit range.
const LOW: u16 = 0;
const HIGH: u16 = 2047;
/// Half a second at 48 MHz.
const READ_PERIOD_CYCLES: u32 = 24_000_000;

static ADC: Mutex<RefCell<Option<Adc>>> = Mutex::new(RefCell::new(None));
/// How many times the handler found the watchdog flag up.
static TRIPS: AtomicU32 = AtomicU32::new(0);

/// Runs `f` on the ADC, which lives with the interrupt handler.
fn with_adc<R>(f: impl FnOnce(&mut Adc) -> R) -> R {
    critical_section::with(|cs| f(ADC.borrow(cs).borrow_mut().as_mut().unwrap()))
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut fmc = dp.fmc.constrain();
    let config = ClockConfig::default()
        .sysclk(SysClk::Pll(PllFreq::Mhz48))
        .adc_sel(AdcSel::Prescaled(AdcPsc::Apb2Div8));
    let mut rcu = dp.rcu.constrain().freeze(&mut fmc, config);

    let gpioa = dp.gpioa.split(&mut rcu);
    let pa4 = gpioa.pa4.into_analog();
    let mut adc = dp.adc.constrain(&mut rcu);

    adc.set_watchdog_window(LOW, HIGH);
    adc.enable_watchdog(&pa4);
    adc.listen(Event::Watchdog);
    critical_section::with(|cs| {
        ADC.borrow(cs).replace(Some(adc));
    });
    // SAFETY: the critical sections here use PRIMASK, not NVIC masks, so
    // unmasking breaks none of them.
    unsafe { NVIC::unmask(pac::Interrupt::ADC_CMP) };

    defmt::info!("watching PA4, window {}..={}", LOW, HIGH);
    loop {
        let before = TRIPS.load(Ordering::Relaxed);
        // A trip raised by this read is handled as soon as the critical
        // section ends, so the count below already includes it.
        let raw = with_adc(|adc| adc.read(&pa4, SampTime::Cycles239_5));
        if TRIPS.load(Ordering::Relaxed) == before {
            defmt::info!("PA4 = {}, in range", raw);
        } else {
            defmt::warn!(
                "PA4 = {}, out of range ({} trips so far)",
                raw,
                TRIPS.load(Ordering::Relaxed)
            );
        }
        cortex_m::asm::delay(READ_PERIOD_CYCLES);
    }
}

#[interrupt]
fn ADC_CMP() {
    with_adc(|adc| {
        if adc.is_pending(Event::Watchdog) {
            adc.clear_interrupt(Event::Watchdog);
            TRIPS.fetch_add(1, Ordering::Relaxed);
        }
    });
}
