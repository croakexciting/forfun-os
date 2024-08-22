use core::arch::asm;
use riscv::register;

pub const PA_VALID_WIDTH: usize = 56;
pub const VA_VALID_WIDTH: usize = 39;
pub const PPN_VALID_WIDTH: usize = 44;
pub const VPN_VALID_WIDTH: usize = 27;
pub const INPAGE_OFFSET_WIDTH: usize = 12;
pub const PAGE_SIZE: usize = 0x1000;
pub const PN_LEVEL_NUM: usize = 3;
pub const PN_BITSIZE: usize = 9;

pub fn root_ppn() -> usize {
    register::satp::read().ppn()
}

pub fn enable_va(id: usize, ppn: usize) {
    let satp = 8usize << 60 | id << 44 | ppn;
    unsafe {
        register::satp::write(satp);
        asm!("sfence.vma");
    }
}