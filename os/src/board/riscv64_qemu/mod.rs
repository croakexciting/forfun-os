pub mod timer;
pub mod interrupt;
pub mod memory;
pub mod peripheral;

use crate::{
    arch::memory::page::kernel_phys_to_virt, driver::{
        self, 
        block::{qemu_blk::QemuBlk, BlkDeviceForFs}, 
    }, file::fs::FILESYSTEM, utils::type_extern::RefCellWrap
};
use alloc::sync::Arc;
use lazy_static::*;
use interrupt::PLIC_ADDR;
use peripheral::{uart_init, UART0_ADDR, BLK_HEADER_ADDR};
use spin::mutex::Mutex;

// 在这里创建一些驱动的单例
lazy_static! {
    pub static ref CONSOLE: RefCellWrap<ns16550a::Uart> = unsafe {
        RefCellWrap::new(uart_init(kernel_phys_to_virt(UART0_ADDR.into()).0))
    };

    pub static ref PLIC: RefCellWrap<driver::ic::plic::PLIC> = unsafe {
        RefCellWrap::new(driver::ic::plic::PLIC::new(kernel_phys_to_virt(PLIC_ADDR.into()).0))
    };
}

pub fn board_init() {
    let blk_dev = BlkDeviceForFs::new(
        Arc::new(Mutex::new(
            QemuBlk::new(kernel_phys_to_virt(BLK_HEADER_ADDR.into()).0)
        )));
    FILESYSTEM.exclusive_access().set_sfs(blk_dev);    
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
    CONSOLE.exclusive_access().put(c as u8);
}

#[allow(deprecated)]
pub fn console_getchar() -> u8 {
    if let Some(c) = CONSOLE.exclusive_access().get() {
        return c;
    } else {
        return 0;
    }
}