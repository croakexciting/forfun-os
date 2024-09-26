pub mod serial;
pub mod memory;
pub mod interrupt;
pub mod timer;

use alloc::sync::Arc;
use lazy_static::*;
use spin::mutex::Mutex;

use crate::{driver::{self, block::{qemu_blk::QemuBlk, BlkDeviceForFs}}, file::fs::FILESYSTEM, process, utils::type_extern::RefCellWrap};

lazy_static! {
    pub static ref CONSOLE: RefCellWrap<arm_pl011::Pl011Uart> = unsafe {
        RefCellWrap::new(serial::init(serial::UART0_ADDR))
    };

    pub static ref GIC: RefCellWrap<driver::ic::gicv2::GICV2> = unsafe {
        RefCellWrap::new(driver::ic::gicv2::GICV2::new(0))
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

// blk0
const BLK_HEADER_ADDR: usize = 0xA00_3E00;

pub fn enable_virtual_mode() {
    let uart_va = process::map_peripheral(serial::UART0_ADDR, 0x1000);
    *CONSOLE.exclusive_access() = serial::init(uart_va as usize);

    let blk_va = process::map_peripheral(BLK_HEADER_ADDR, 0x1000);
    let blk_device = BlkDeviceForFs::new(Arc::new(Mutex::new(QemuBlk::new(blk_va as usize))));
    FILESYSTEM.exclusive_access().set_sfs(blk_device);

    let gic_va = process::map_peripheral(0x800_0000, 0x2_0000);
    GIC.exclusive_access().set_addr(gic_va as usize);
}

pub fn board_init() {
    CONSOLE.exclusive_access().init();
    CONSOLE.exclusive_access().ack_interrupts();
    CONSOLE.exclusive_access().is_receive_interrupt();
    GIC.exclusive_access().enable(30);
    GIC.exclusive_access().set_priority(255);
}

