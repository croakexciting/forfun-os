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
use process::{create_app, run_apps};
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

    let elf1 = unsafe { core::slice::from_raw_parts(0x8100_0000 as *mut u8, 4096*100)};
    let app1 = create_app(elf1);
    if app1 < 0 {
        panic!("create app1 with return code {}", app1);
    }

    let elf2 = unsafe { core::slice::from_raw_parts(0x8200_0000 as *mut u8, 4096*100)};
    let app2 = create_app(elf2);
    if app2 < 0 {
        panic!("create app2 with return code {}", app2);
    }

    let elf3 = unsafe { core::slice::from_raw_parts(0x8300_0000 as *mut u8, 4096*100)};
    let app3 = create_app(elf3);
    if app3 < 0 {
        panic!("create app3 with return code {}", app3);
    }

    let elf4 = unsafe { core::slice::from_raw_parts(0x8400_0000 as *mut u8, 4096*100)};
    let app4 = create_app(elf4);
    if app4 < 0 {
        panic!("create app3 with return code {}", app4);
    }

    let elf5 = unsafe { core::slice::from_raw_parts(0x8500_0000 as *mut u8, 4096*100)};
    let app5 = create_app(elf5);
    if app5 < 0 {
        panic!("create app3 with return code {}", app5);
    }

    run_apps();
}
