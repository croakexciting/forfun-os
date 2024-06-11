#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(error_in_core)]

mod arch;
mod sbi;
#[macro_use]
mod utils;
mod trap;
mod syscall;
mod process;
mod config;
mod mm;
mod driver;

#[cfg(feature = "riscv_qemu")]
#[path = "board/riscv_qemu.rs"]
mod board;

use core::arch::global_asm;
extern crate alloc;
use process::{activate_app, create_app, run_apps};
use buddy_system_allocator::LockedHeap;
use utils::timer;

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
    trap::enable_timer_interrupt();
    timer::set_trigger();

    let elf = unsafe { core::slice::from_raw_parts(0x8100_0000 as *mut u8, 4096*100)};
    let app = create_app(elf);
    if app >= 0 {
        activate_app(app as usize);
    } else {
        println!("create app with return code {}", app);
    }

    run_apps();
}
