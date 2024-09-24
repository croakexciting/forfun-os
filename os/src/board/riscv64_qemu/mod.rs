pub mod timer;
pub mod interrupt;
pub mod serial;
pub mod memory;

use crate::{
    driver::{self, block::{qemu_blk::QemuBlk, BlkDeviceForFs}, serial::qemu_serial::Uart}, file::fs::FILESYSTEM, process, utils::type_extern::RefCellWrap
};
use alloc::sync::Arc;
use lazy_static::*;
use interrupt::PLIC_ADDR;
use spin::mutex::Mutex;

// 在这里创建一些驱动的单例

lazy_static! {
    pub static ref CONSOLE: RefCellWrap<Uart> = unsafe {
        RefCellWrap::new(serial::init())
    };

    pub static ref PLIC: RefCellWrap<driver::ic::plic::PLIC> = unsafe {
        RefCellWrap::new(driver::ic::plic::PLIC::new(0))
    };
}

// blk0
const BLK_HEADER_ADDR: usize = 0x1000_8000;

pub fn enable_virtual_mode() {
    let uart_va = process::map_peripheral(serial::UART0_ADDR, 0x1000);
    CONSOLE.exclusive_access().set_addr(uart_va as usize);

    let blk_va = process::map_peripheral(BLK_HEADER_ADDR, 0x8000);
    // create fs in here
    let blk_device = BlkDeviceForFs::new(Arc::new(Mutex::new(QemuBlk::new(blk_va as usize))));
    FILESYSTEM.exclusive_access().set_sfs(blk_device);

    let plic_va = process::map_peripheral(PLIC_ADDR, 0x40_0000);
    PLIC.exclusive_access().set_addr(plic_va as usize)
}

pub fn board_init() {
    interrupt::plic_init();
}

pub fn shutdown(failure: bool) -> ! {
    use sbi_rt::{system_reset, NoReason, Shutdown, SystemFailure};
    if !failure {
        system_reset(Shutdown, NoReason);
    } else {
        system_reset(Shutdown, SystemFailure);
    }
    unreachable!()
}

pub fn console_putchar(c: char) {
    CONSOLE.exclusive_access().put(c);
}

#[allow(deprecated)]
pub fn console_getchar() -> u8 {
    sbi_rt::legacy::console_getchar() as u8
}