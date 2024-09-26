use aarch64_cpu::registers::*;

pub const CLOCK_FREQ: usize = 62500000;

pub fn nanoseconds() -> usize {
    CNTPCT_EL0.get() as usize * 1_000_000_000 / CLOCK_FREQ
}

pub fn set_trigger(tick_per_sec: usize) {
    let cntpct_el0 = CNTPCT_EL0.get() as usize;
    let new_tick = (cntpct_el0 + CLOCK_FREQ / tick_per_sec) as u64;

    CNTP_CVAL_EL0.set(new_tick);
}