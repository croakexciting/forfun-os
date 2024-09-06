pub mod trap;
pub mod context;
pub mod memory;

use core::{arch::global_asm, hint::spin_loop};
global_asm!(include_str!("entry.S"));

pub fn shutdown(_failure: bool) -> ! {
    spin_loop();
    unreachable!()
}