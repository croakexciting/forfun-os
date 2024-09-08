use core::arch::asm;

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

    unsafe {
        asm!(
            "msr CNTP_CVAL_EL0, {}",
            in(reg) new_tick
        )
    }
}