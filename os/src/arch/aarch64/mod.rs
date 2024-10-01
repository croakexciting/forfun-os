pub mod trap;
pub mod context;
pub mod memory;
pub mod trampoline;

use core::arch::global_asm;

global_asm!(include_str!("switch.S"));
