use crate::{driver::{self, block::qemu_blk::QemuBlk, serial::Uart}, process, utils::type_extern::RefCellWrap};

pub mod timer;
pub mod plic;
pub mod serial;

use lazy_static::*;

// 在这里创建一些驱动的单例

lazy_static! {
    pub static ref CONSOLE: RefCellWrap<Uart> = unsafe {
        RefCellWrap::new(serial::init())
    };

    // pub static ref PLIC: RefCellWrap<driver::plic::qemu_plic::PLIC> = unsafe {
    //     RefCellWrap::new(plic::plic_init())
    // };
}

pub fn enable_virtual_mode() {
    let uart_va = process::map_peripheral(serial::UART0_ADDR, 0x1000);
    CONSOLE.exclusive_access().set_addr(uart_va as usize);
}

// blk0
const BLK_HEADER_ADDR: usize = 0x1000_8000;

pub fn init_blk0() -> QemuBlk {
    let blk_va = process::map_peripheral(BLK_HEADER_ADDR, 0x1000);
    QemuBlk::new(blk_va as usize)
}