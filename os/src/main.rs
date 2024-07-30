#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(error_in_core)]
#![feature(alloc_error_handler)]
#![feature(const_slice_from_raw_parts_mut)]

mod arch;
mod sbi;
#[macro_use]
mod utils;
mod trap;
mod syscall;
mod process;
mod mm;
mod driver;
mod file;
mod ipc;

#[cfg(feature = "riscv_qemu")]
#[path = "board/riscv_qemu.rs"]
mod board;

use core::arch::global_asm;
extern crate alloc;
use process::{create_proc, run_tasks};
use linked_list_allocator::LockedHeap;
use utils::timer;

global_asm!(include_str!("arch/riscv64/entry.asm"));
global_asm!(include_str!("trap/trap.S"));
global_asm!(include_str!("process/switch.S"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
/// panic when heap allocation error occurs
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

pub fn init_heap() {
    extern "C" {
        fn sheap();
        fn eheap();
    }

    println!("[kernel] heap start at {:#x}, end at {:#x}", sheap as usize, eheap as usize);
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(sheap as usize as *mut u8, eheap as usize - sheap as usize);
    }
}

#[no_mangle]
pub fn os_main(_hart_id: usize, _dts: usize) -> ! {
    println!("[kernel] hart id is: {}, dts addr is {:#x}", _hart_id, _dts);
    clear_bss();
    init_heap();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_trigger();
    create_proc();
    run_tasks();
}
