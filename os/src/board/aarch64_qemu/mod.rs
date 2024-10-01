pub mod memory;
pub mod interrupt;
pub mod timer;
pub mod peripheral;

use alloc::sync::Arc;
use lazy_static::*;
use spin::mutex::Mutex;

use crate::{
    arch::memory::page::kernel_phys_to_virt, 
    driver::{self, block::{qemu_blk::QemuBlk, BlkDeviceForFs}}, 
    file::fs::FILESYSTEM, utils::type_extern::RefCellWrap
};

lazy_static! {
    pub static ref CONSOLE: RefCellWrap<arm_pl011::Pl011Uart> = unsafe {
        RefCellWrap::new(peripheral::init_serial(
            kernel_phys_to_virt(peripheral::UART0_ADDR.into()).0
        ))
    };

    pub static ref GIC: RefCellWrap<driver::ic::gicv2::GICV2> = unsafe {
        RefCellWrap::new(driver::ic::gicv2::GICV2::new(
            kernel_phys_to_virt(interrupt::GIC_ADDR.into()).0
        ))
    };
}

pub fn shutdown(_failure: bool) -> ! {
    loop {}
}

pub fn console_putchar(c: char) {
    CONSOLE.exclusive_access().putchar(c as u8)
}

pub fn console_getchar() -> u8 {
    if let Some(c) = CONSOLE.exclusive_access().getchar() {
        return c;
    } else {
        return 0;
    }
}

pub fn board_init() {
    CONSOLE.exclusive_access().init();
    CONSOLE.exclusive_access().ack_interrupts();
    CONSOLE.exclusive_access().is_receive_interrupt();

    GIC.exclusive_access().enable(30);
    GIC.exclusive_access().set_priority(255);

    let blk_device = BlkDeviceForFs::new(
        Arc::new(Mutex::new(QemuBlk::new(
            kernel_phys_to_virt(peripheral::BLK_HEADER_ADDR.into()).0
        ))));
    FILESYSTEM.exclusive_access().set_sfs(blk_device);
}

