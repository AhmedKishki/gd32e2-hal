//! Embassy's time driver on TIMER2.
//!
//! The timer counts at Embassy's tick rate, 1 MHz unless the program enables
//! another `tick-hz-*` feature of `embassy-time`. The TIMER2 interrupt extends
//! the 16-bit counter to 64 bits by counting its half-wraps, one every 32,768
//! ticks. Ported from `embassy-stm32`'s `src/time_driver/tim.rs`.
//!
//! Copyright (c) Embassy project contributors