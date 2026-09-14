//! System configuration: the registers that wire peripherals to each other
//! rather than to the outside world.
//!
//! Covers the boot mode and `EXTISS`, the latter through
//! [`ExtiLine::source`](crate::exti::ExtiLine::source).

use crate::pac;
use crate::rcu::Rcu;

/// The memory the chip booted from (`BOOT_MODE`).
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BootMode {
    /// Main flash: the application.
    MainFlash,
    /// System flash memory: the factory bootloader.
    SystemFlash,
    /// Embedded SRAM.
    Sram,
}

/// Owns the SYSCFG registers.
pub struct Syscfg {
    syscfg: pac::Syscfg,
}

impl Syscfg {
    /// Returns the peripheral. The clock stays on, since `CFGCMPEN` also feeds
    /// the comparator.
    pub fn release(self) -> pac::Syscfg {
        self.syscfg
    }

    /// Returns the memory the chip booted from, as latched at the last reset.
    pub fn boot_mode(&self) -> BootMode {
        match self.syscfg.cfg0().read().boot_mode().bits() {
            0b00 | 0b10 => BootMode::MainFlash,
            0b01 => BootMode::SystemFlash,
            0b11 => BootMode::Sram,
            _ => unreachable!(),
        }
    }

    /// Points EXTI line `line` at the port encoded by `code`.
    ///
    /// # Panics
    ///
    /// If `line` is above 15 — the lines above that have no source to select.
    pub(crate) fn set_extiss(&mut self, line: u8, code: u8) {
        match line {
            // SAFETY: `code` is an `ExtiPin::SS_CODE`, one of the port codes this
            // field takes.
            l @ 0..=3 => self.syscfg.extiss0().modify(|_, w| unsafe {
                match l {
                    0 => w.exti0_ss().bits(code),
                    1 => w.exti1_ss().bits(code),
                    2 => w.exti2_ss().bits(code),
                    _ => w.exti3_ss().bits(code),
                }
            }),
            // SAFETY: as above.
            l @ 4..=7 => self.syscfg.extiss1().modify(|_, w| unsafe {
                match l {
                    4 => w.exti4_ss().bits(code),
                    5 => w.exti5_ss().bits(code),
                    6 => w.exti6_ss().bits(code),
                    _ => w.exti7_ss().bits(code),
                }
            }),
            // SAFETY: as above.
            l @ 8..=11 => self.syscfg.extiss2().modify(|_, w| unsafe {
                match l {
                    8 => w.exti8_ss().bits(code),
                    9 => w.exti9_ss().bits(code),
                    10 => w.exti10_ss().bits(code),
                    _ => w.exti11_ss().bits(code),
                }
            }),
            // SAFETY: as above.
            l @ 12..=15 => self.syscfg.extiss3().modify(|_, w| unsafe {
                match l {
                    12 => w.exti12_ss().bits(code),
                    13 => w.exti13_ss().bits(code),
                    14 => w.exti14_ss().bits(code),
                    _ => w.exti15_ss().bits(code),
                }
            }),
            _ => unreachable!(),
        }
    }
}

/// Takes the raw peripheral into [`Syscfg`].
pub trait SyscfgExt {
    /// Switches the SYSCFG clock on and hands back the owning type.
    ///
    /// The clock bit is shared with the comparator, so CMP is clocked too; see
    /// [`Rcu::enable_cfgcmp`].
    fn constrain(self, rcu: &mut Rcu) -> Syscfg;
}

impl SyscfgExt for pac::Syscfg {
    fn constrain(self, rcu: &mut Rcu) -> Syscfg {
        rcu.enable_cfgcmp();
        Syscfg { syscfg: self }
    }
}
