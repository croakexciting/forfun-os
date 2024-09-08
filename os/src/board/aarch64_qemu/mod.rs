pub mod serial;
pub mod memory;
pub mod plic;
pub mod timer;

use alloc::sync::Arc;
use lazy_static::*;
use spin::RwLock;

use crate::{process, utils::type_extern::RefCellWrap};

lazy_static! {
    pub static ref CONSOLE: RefCellWrap<arm_pl011::Pl011Uart> = unsafe {
        RefCellWrap::new(serial::init(serial::UART0_ADDR))
    };

    pub static ref BLK0_VA: Arc<RwLock<usize>> = {
        Arc::new(RwLock::new(0))
    };
}

pub fn shutdown(_failure: bool) -> ! {
    unreachable!()
}

pub fn console_putchar(c: char) {
    CONSOLE.exclusive_access().putchar(c as u8)
}

pub fn console_getchar() -> u8 {
    CONSOLE.exclusive_access().getchar().unwrap()
}

// blk0
const BLK_HEADER_ADDR: usize = 0x1000_8000;

pub fn enable_virtual_mode() {
    let uart_va = process::map_peripheral(serial::UART0_ADDR, 0x1000);
     *CONSOLE.exclusive_access() = serial::init(uart_va as usize);

     let blk_va = process::map_peripheral(BLK_HEADER_ADDR, 0x8000);
     *BLK0_VA.write() = blk_va as usize;
}

pub fn board_init() {}

