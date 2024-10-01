use core::arch::asm;
use bitflags::bitflags;
use riscv::register;
use crate::arch::memory::page::PTEFlags;

pub const VA_VALID_WIDTH: usize = 39;
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

bitflags! {
    #[derive(Clone, Copy)]
    pub struct RiscvPteFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

pub fn pte(ppn: usize, flags: PTEFlags) -> usize {
    let mut riscv_flags = RiscvPteFlags::empty();
    riscv_flags.insert(RiscvPteFlags::V);
    if flags.contains(PTEFlags::U) {
        riscv_flags.insert(RiscvPteFlags::U);
    } 
    
    if flags.contains(PTEFlags::R) {
        riscv_flags.insert(RiscvPteFlags::R)
    } 
    
    if flags.contains(PTEFlags::W) {
        riscv_flags.insert(RiscvPteFlags::W)
    } 
    
    if flags.contains(PTEFlags::X) {
        riscv_flags.insert(RiscvPteFlags::X)
    }

    ppn << 10 | riscv_flags.bits() as usize
}

pub fn ppn(pte: usize) -> usize {
    pte >> 10
}

pub fn flags(pte: usize) -> Option<PTEFlags> {
    let riscv_flags = RiscvPteFlags::from_bits(pte as u8)?;
    let mut flags = PTEFlags::empty();
    if riscv_flags.contains(RiscvPteFlags::V) {
        flags.insert(PTEFlags::V);
        flags.insert(PTEFlags::T);
    }

    if riscv_flags.contains(RiscvPteFlags::R) {
        flags.insert(PTEFlags::R);
        flags.remove(PTEFlags::T);
    }

    if riscv_flags.contains(RiscvPteFlags::W) {
        flags.insert(PTEFlags::W);
        flags.remove(PTEFlags::T);
    }

    if riscv_flags.contains(RiscvPteFlags::X) {
        flags.insert(PTEFlags::X);
        flags.remove(PTEFlags::T);
    }

    if riscv_flags.contains(RiscvPteFlags::U) {
        flags.insert(PTEFlags::U);
    }

    Some(flags)
}

pub fn is_set(pte: usize, flags: PTEFlags) -> bool {
    let mut result = false;
    if let Some(riscv_flags) = RiscvPteFlags::from_bits(pte as u8) {
        if flags.contains(PTEFlags::V) || flags.contains(PTEFlags::T) {
            if riscv_flags.contains(RiscvPteFlags::V) {
                result = true;
            }
        }

        if flags.contains(PTEFlags::R) {
            if riscv_flags.contains(RiscvPteFlags::R) {
                result = true;
            }
        }

        if flags.contains(PTEFlags::W) {
            if riscv_flags.contains(RiscvPteFlags::W) {
                result = true;
            }
        }

        if flags.contains(PTEFlags::X) {
            if riscv_flags.contains(RiscvPteFlags::X) {
                result = true;
            }
        }

        if flags.contains(PTEFlags::U) {
            if riscv_flags.contains(RiscvPteFlags::U) {
                result = true;
            }
        }
    }

    result
}

pub fn set_flag(pte: usize, flags: PTEFlags) -> usize {
    let mut p = pte;
    if let Some(mut riscv_flags) = RiscvPteFlags::from_bits(p as u8) {
        if flags.contains(PTEFlags::U) {
            riscv_flags.insert(RiscvPteFlags::U);
        } 
        
        if flags.contains(PTEFlags::R) {
            riscv_flags.insert(RiscvPteFlags::R);
        } 
        
        if flags.contains(PTEFlags::W) {
            riscv_flags.insert(RiscvPteFlags::W);
        } 
        
        if flags.contains(PTEFlags::X) {
            riscv_flags.insert(RiscvPteFlags::X);
        }

        p |= riscv_flags.bits() as usize;
    }

    p
}

pub fn clear_flag(pte: usize, flags: PTEFlags) -> usize {
    let mut p = pte;
    if let Some(mut riscv_flags) = RiscvPteFlags::from_bits(p as u8) {
        if flags.contains(PTEFlags::U) {
            riscv_flags.remove(RiscvPteFlags::U);
        } 
        
        if flags.contains(PTEFlags::R) {
            riscv_flags.remove(RiscvPteFlags::R);
        } 
        
        if flags.contains(PTEFlags::W) {
            riscv_flags.remove(RiscvPteFlags::W);
        } 
        
        if flags.contains(PTEFlags::X) {
            riscv_flags.remove(RiscvPteFlags::X);
        }

        let mask = 0xFFFFFFFFFFFFFF00 | riscv_flags.bits() as usize;
        p &= mask;
    }

    p
}

pub unsafe fn flush_tlb(_asid: usize) {
    asm!("sfence.vma");
}