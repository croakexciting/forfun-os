pub mod trap;
pub mod context;
pub mod config;

use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));

pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}
