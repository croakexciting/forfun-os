use crate::{driver, utils::type_extern::RefCellWrap};

pub mod timer;
pub mod plic;

use lazy_static::*;

lazy_static! {
    pub static ref PERIPHERAL: RefCellWrap<Peripheral> = unsafe {
        RefCellWrap::new(v)
    }
}

pub struct Peripheral {
    console: driver::serial::Uart,
    plic: driver::plic::qemu_plic::PLIC,
    blk: driver::block::qemu_blk::QemuBlk,
}

impl Peripheral {
    pub fn init() -> Self {
        let plic = plic::plic_init();
    }
}
