use crate::{driver::{self, serial::Uart}, utils::type_extern::RefCellWrap};

pub mod timer;
pub mod plic;
pub mod serial;

use alloc::sync::Arc;
use lazy_static::*;
use spin::rwlock::RwLock;

// lazy_static! {
//     pub static ref PERIPHERAL: RefCellWrap<Peripheral> = unsafe {
//         RefCellWrap::new(v)
//     }
// }

// pub struct Peripheral {
//     console: driver::serial::Uart,
//     plic: driver::plic::qemu_plic::PLIC,
//     blk: driver::block::qemu_blk::QemuBlk,
// }

// 在这里创建一些驱动的单例

lazy_static! {
    pub static ref CONSOLE: RefCellWrap<Uart> = unsafe {
        RefCellWrap::new(Uart::new(0x1000_0000))
    };
}

// impl Peripheral {
//     pub fn init() -> Self {
//         let plic = plic::plic_init();
//         let uart = driver::serial::Uart
        
//     }
// }
