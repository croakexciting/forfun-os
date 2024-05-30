#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod arch;
mod sbi;
mod utils;

use core::arch::global_asm;

global_asm!(include_str!("arch/riscv64/entry.asm"));

#[no_mangle]
pub fn os_main() -> ! {
    println!("Hello, world!");
    sbi::shutdown(false)
}
