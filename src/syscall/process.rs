use crate::process::{exit, sleep};

pub fn sys_exit(code: i32) -> ! {
    exit(code);
}

pub fn sys_yield() {
    sleep(0);
}

pub fn sys_nanosleep(duration: usize) {
    sleep(duration)
}