#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![feature(error_in_core)]
#![feature(alloc_error_handler)]
#![feature(const_slice_from_raw_parts_mut)]
#![feature(naked_functions)]

mod arch;
mod board;

#[macro_use]
mod utils;

mod driver;
mod file;
mod ipc;
mod mm;
mod process;
mod syscall;

extern crate alloc;
use board::board_init;
use linked_list_allocator::LockedHeap;
use process::{create_proc, run_tasks};
use crate::board::timer;

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

    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(sheap as usize as *mut u8, eheap as usize - sheap as usize);
    }
}

#[no_mangle]
#[link_section = ".text.entry"]
pub fn os_main() -> ! {
    clear_bss();
    init_heap();
    arch::init();
    timer::set_trigger();
    board_init();
    create_proc();
    run_tasks();
}
