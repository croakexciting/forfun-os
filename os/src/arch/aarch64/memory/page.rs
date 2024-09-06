use core::arch::asm;
use aarch64_cpu::registers::*;
use tock_registers::interfaces::ReadWriteable;

pub const PAGE_SIZE: usize = 0x1000;
pub const PN_BITSIZE: usize = 9;
pub const PN_LEVEL_NUM: usize = 4;
pub const INPAGE_OFFSET_WIDTH: usize = 12;
pub const PA_VALID_WIDTH: usize = 48;
pub const VA_VALID_WIDTH: usize = 39;

// arm64 physical page width set to 36
pub const PPN_VALID_WIDTH: usize = 36;
// use three level pte
pub const VPN_VALID_WIDTH: usize = 27;

pub fn root_ppn() -> usize {
    aarch64_cpu::registers::TTBR0_EL1.get_baddr() as usize
}

pub fn enable_va(id: usize, ppn: usize) {
    let ttbr0_el1 = (id << 56) | (ppn & 0x0000_FFFF_FFFF_F000);
    unsafe {
        asm!(
            "msr ttbr0_el1, {0}",
            in(reg) ttbr0_el1,
            options(nostack, preserves_flags),
        );
        SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
    }
}