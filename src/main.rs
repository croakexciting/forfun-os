#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod arch;
mod sbi;
#[macro_use]
mod utils;
mod trap;
mod syscall;
mod process;
mod config;

#[cfg(feature = "riscv_qemu")]
#[path = "board/riscv_qemu.rs"]
mod board;

use core::arch::global_asm;
extern crate alloc;
use process::{create_app, run_apps};
use buddy_system_allocator::LockedHeap;

global_asm!(include_str!("arch/riscv64/entry.asm"));

#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

fn init_heap() {
    extern "C" {
        fn sheap();
        fn _heap_size();
    }

    unsafe {
        HEAP_ALLOCATOR.lock().init(sheap as usize, _heap_size as usize);
    }
}

#[no_mangle]
pub fn os_main() -> ! {
    init_heap();
    trap::init();

    create_app(0x80400000);
    create_app(0x80300000);
    create_app(0x80200000);
    run_apps();
}
