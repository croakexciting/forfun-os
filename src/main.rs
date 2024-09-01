#![no_std]
#![no_main]
#![feature(panic_info_message)]

mod arch;
#[macro_use]
mod utils;
mod syscall;
mod process;
mod config;

extern crate alloc;
use process::{start_first_app, create_app};
use linked_list_allocator::LockedHeap;

#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init_heap() {
    extern "C" {
        fn sheap();
        fn eheap();
    }

    println!(
        "[kernel] heap start at {:#x}, end at {:#x}",
        sheap as usize, eheap as usize
    );
    unsafe {
        HEAP_ALLOCATOR
            .lock()
            .init(sheap as usize as *mut u8, eheap as usize - sheap as usize);
    }
}

#[no_mangle]
pub fn os_main() -> ! {
    init_heap();
    println!("kernel start");
    arch::init();

    start_first_app();
}
