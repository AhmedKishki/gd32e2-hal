//! Time driver for `embassy-time`, on TIMER2.
//!
//! `embassy-time` (`Timer::after`, `Ticker`, `with_timeout`) reads the time
//! from a driver in the HAL. A program that uses `embassy-time` without a
//! driver fails to link.
//!
//! TIMER2 counts at the tick rate: 1 MHz, unless the program's `Cargo.toml`
//! enables another `tick-hz-*` feature of `embassy-time`. Its counter has 16
//! bits, so at 1 MHz it wraps every 65.536 ms. At counts 0 and 32,768, the
//! TIMER2 interrupt handler adds one to a 32-bit count of half-wraps. A
//! half-wrap is 32,768 (2^15) counts, so the time in ticks has 32 + 15 = 47
//! bits. It overflows after 2^47 ticks: 4.46 years at 1 MHz, 2.23 years at
//! 2 MHz.
//!
//! The code is a port of `embassy-stm32`'s `src/time_driver/tim.rs`, which
//! uses the STM32 general-purpose timers. Each register the driver uses has an
//! STM32 counterpart; six of them have other names: CR1 is CTL0, DIER is
//! DMAINTEN, SR is INTF, EGR is SWEVG, ARR is CAR and CCR1 is CH0CV.
//!
//! Copyright (c) Embassy project contributors

use embassy_time_driver::TICK_HZ;

use crate::pac;
use crate::rcu::{Enable, Rcu, Reset};
use crate::timer::{ChannelEnable, Instance};

/// Starts TIMER2 counting at the tick rate.
///
/// Call it in `main`, after `freeze` sets the clocks and before the first
/// `embassy-time` call:
///
/// ```ignore
/// use gd32e2_hal::{pac, prelude::*, rcu::ClockConfig, time_driver};
///
/// let dp = pac::Peripherals::take().unwrap();
/// let mut fmc = dp.fmc.constrain();
/// let mut rcu = dp.rcu.constrain().freeze(&mut fmc, ClockConfig::default());
/// time_driver::init(dp.timer2, &mut rcu);
/// ```
///
/// TIMER2's input clock is the APB1 timer clock,
/// [`Clocks::pclk1_tim`](crate::rcu::Clocks::pclk1_tim), which `freeze` sets
/// from its `ClockConfig`. TIMER2's prescaler divides that clock to the tick
/// rate: by 8 for the default config (8 MHz) and the default tick rate
/// (1 MHz), and by 72 for a 72 MHz clock.
///
/// # Panics
///
/// `init` panics if the clock is not a whole multiple of the tick rate
/// (1.125 MHz with a 1 MHz tick), or if the divider would exceed 65,536
/// (72 MHz with `tick-hz-1_000` needs 72,000).
pub fn init(mut timer: pac::Timer2, rcu: &mut Rcu) {
    // Turn the clock on, then reset TIMER2, so every register that `init` does
    // not write holds its reset value, whatever code used TIMER2 before.
    pac::Timer2::enable(rcu);
    pac::Timer2::reset(rcu);

    // `init` panics instead of rounding the divider: with a 1.125 MHz clock
    // and a 1 MHz tick, a divider rounded to 1 would make every delay end 11 %
    // early. `clk` is a u64, the type of `TICK_HZ`; widening loses no bits.
    // PSC holds the divider minus 1.
    let clk = u64::from(pac::Timer2::clk(&rcu.clocks()).to_Hz());
    assert!(clk % TICK_HZ == 0, "TIMER2 clock not a multiple of TICK_HZ");
    let Ok(psc) = u16::try_from(clk / TICK_HZ - 1) else {
        panic!("TIMER2 clock too fast for TICK_HZ");
    };
    timer.set_psc(psc);

    // CAR, the top count, resets to 0, so `init` must write it. 65,535 makes
    // CNT wrap every 65,536 (2^16) counts, so each half-wrap is exactly 32,768
    // (2^15) counts, as the half-wrap count needs.
    timer.set_car(u16::MAX);

    // A new PSC value takes effect only at an update event. UPG generates one
    // now, so CNT starts from 0 at the tick rate. With UPS set, this update
    // event leaves UPIF clear, so the interrupt handler counts no half-wrap
    // for it.
    timer.set_ups(true);
    timer.gen_update();
    timer.set_ups(false);

    // Channel 0 compares CNT with 32,768 (0x8000) and sets CH0IF half-way
    // between wraps. It drives no pin.
    // SAFETY: every u16 is a legal compare value.
    timer.ch0cv().write(|w| unsafe { w.ch0val().bits(0x8000) });

    // Request the TIMER2 interrupt when CNT wraps to 0 (UPIF) and when it
    // equals 32,768 (CH0IF): every 32.768 ms at 1 MHz.
    timer.set_upie(true);
    ChannelEnable::<0>::set_chxie(&mut timer, true);

    // Start CNT last, after UPG has made PSC take effect, so the first count
    // already takes one tick.
    timer.set_cen(true);
}
