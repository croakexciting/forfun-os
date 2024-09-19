use riscv::register::time;
pub const CLOCK_FREQ: usize = 12500000;

pub fn set_timer(time: u64) {
    sbi_rt::set_timer(time);
}

pub fn nanoseconds() -> usize {
    (time::read() * 1_000_000_000) / CLOCK_FREQ
}

pub fn set_trigger(ticks_per_sec: usize) {
    set_timer((time::read() + CLOCK_FREQ / ticks_per_sec) as u64)
}