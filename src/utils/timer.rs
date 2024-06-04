#![allow(unused)]

use crate::board::CLOCK_FREQ;
use crate::sbi::set_timer;
use riscv::register::time;

// 1ms for every tick
const TICKS_PER_SEC: usize = 1000;

pub fn nanoseconds() -> usize {
    (time::read() * 1_000_000_000) / CLOCK_FREQ
}

pub fn set_next_trigger() {
    set_timer((time::read() + CLOCK_FREQ / TICKS_PER_SEC) as u64)
}