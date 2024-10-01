pub mod context;
pub mod trap;
pub mod memory;
pub mod trampoline;

use core::arch::global_asm;

global_asm!(include_str!("trap.S"));
global_asm!(include_str!("switch.S"));
