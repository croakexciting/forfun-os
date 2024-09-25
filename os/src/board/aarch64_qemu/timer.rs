use core::arch::asm;
use aarch64_cpu::{asm::barrier, registers::*};
use tock_registers::interfaces::ReadWriteable;

pub const CLOCK_FREQ: usize = 62500000;

pub fn nanoseconds() -> usize {
    let mut cntpct_el0: usize;
    unsafe {
        asm!(
            "mrs {0}, CNTPCT_EL0",
            out(reg) cntpct_el0
        );
    }

    cntpct_el0 * 1_000_000_000 / CLOCK_FREQ
}

pub fn set_trigger(tick_per_sec: usize) {
    let mut cntpct_el0: usize;
    unsafe {
        asm!(
            "mrs {0}, CNTPCT_EL0",
            out(reg) cntpct_el0
        );
    }

    let new_tick = (cntpct_el0 + CLOCK_FREQ / tick_per_sec) as u64;

    CNTP_CVAL_EL0.set(new_tick);
    CNTP_CTL_EL0.modify(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR);
}