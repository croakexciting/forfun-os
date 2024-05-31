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

use core::arch::global_asm;
use trap::context::TrapContext;
use buddy_system_allocator::LockedHeap;
use once_cell::sync::OnceCell;

global_asm!(include_str!("arch/riscv64/entry.asm"));

#[global_allocator]
/// heap allocator instance
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();
static MY_ONCE_CELL: OnceCell<u32> = OnceCell::new();

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
    MY_ONCE_CELL.set(42).unwrap();
    println!("Hello, world!");
    println!("trap context struct size {}", core::mem::size_of::<TrapContext>());
    sbi::shutdown(false)
}
