use core::arch::asm;
use aarch64_cpu::registers::*;
use tock_registers::interfaces::ReadWriteable;

use crate::arch::memory::page::PTEFlags;

pub const PAGE_SIZE: usize = 0x1000;
pub const PN_BITSIZE: usize = 9;
pub const PN_LEVEL_NUM: usize = 4;
pub const INPAGE_OFFSET_WIDTH: usize = 12;
pub const PA_VALID_WIDTH: usize = 32;
pub const VA_VALID_WIDTH: usize = 39;

// arm64 physical page width set to 36
pub const PPN_VALID_WIDTH: usize = 20;
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

pub fn pte(ppn: usize, flags: PTEFlags) -> usize {
    let mut pte: usize = 0;
    // set valid
    pte |= 0x1;

    // table or block
    if flags.contains(PTEFlags::T) {
        pte |= (0x1 << 1) as usize
    }

    // read/write permissions
    if flags.contains(PTEFlags::U) {
        if flags.contains(PTEFlags::R) {
            if flags.contains(PTEFlags::W) {
                pte |= (0b10 << 6) as usize
            } else {
                pte |= (0b01 << 6) as usize
            }
        }
    } else {
        if flags.contains(PTEFlags::R) && !flags.contains(PTEFlags::W) {
            pte |= (0b11 << 6) as usize
        }
    }

    // add ppn
    let mppn = ppn &  0x0000_0000_000F_FFFF;
    pte |= (mppn << 12) as usize;

    // add execute permissions
    if flags.contains(PTEFlags::X) {
        if flags.contains(PTEFlags::U) {
            pte |= (0usize << 53) as usize
        } else {
            pte |= (2usize << 53) as usize
        }
    } else {
        pte |= (3usize << 53) as usize
    }

    pte
}

pub fn ppn(pte: usize) -> usize {
    (pte >> 12) & 0x0000_0000_000F_FFFF
}

pub fn flags(pte: usize) -> Option<PTEFlags> {
    let mut flags = PTEFlags::empty();
    if pte & 1usize > 0 {
        flags.insert(PTEFlags::V)
    }

    if pte & (1usize << 1) > 0 {
        flags.insert(PTEFlags::T);
    }

    let ap = (pte & (3usize << 6)) >> 6;
    if ap == 0 {
        flags.insert(PTEFlags::R | PTEFlags::W)
    } else if ap == 1 {
        flags.insert(PTEFlags::R | PTEFlags::U)
    } else if ap == 2 {
        flags.insert(PTEFlags::R | PTEFlags::W | PTEFlags::U)
    } else if ap == 3 {
        flags.insert(PTEFlags::R)
    }

    let x = (pte & 3usize << 53) >> 53;
    if x == 0 {
        flags.insert(PTEFlags::X | PTEFlags::U)
    } else if x == 2 {
        flags.insert(PTEFlags::X)
    }

    Some(flags)
}

pub fn is_set(pte: usize, flags: PTEFlags) -> bool {
    let mut result = false;
    if flags.contains(PTEFlags::V) || flags.contains(PTEFlags::R) {
        if pte & 1usize > 0 {
            result = true;
        }
    }

    if flags.contains(PTEFlags::T) {
        if pte & (1usize << 1) > 0 {
            result = true
        }
    }

    if flags.contains(PTEFlags::W) {
        let ap = (pte & (3usize << 6)) >> 6;
        if ap == 0 || ap == 2 {
            result = true
        }
    }

    // if flags.contains(PTEFlags::U) {
    //     let ap = (pte & (3usize << 6)) >> 6;
    //     let x = (pte & 3usize << 53) >> 53;
    //     if (ap == 1 || ap == 2) && x == 0 {
    //         result = true
    //     }
    // }

    if flags.contains(PTEFlags::X) {
        let x = (pte & 3usize << 53) >> 53;
        if x != 3 {
            result = true
        }
    }

    result
}

pub fn set_flag(pte: usize, f: PTEFlags) -> usize {
    let mut p = pte;
    let current_flags = flags(pte).unwrap();
    
    if f.contains(PTEFlags::V) {
        p |= 0x1;
    }

    if f.contains(PTEFlags::T) {
        p |= (0x1 << 1) as usize
    }

    if f.contains(PTEFlags::W) {
        if !current_flags.contains(PTEFlags::U) {
            p &= !((0b11 << 6) as usize);
        } else {
            p &= !((0b11 << 6) as usize);
            p |= (0b10 << 6) as usize;
        }
    }

    if f.contains(PTEFlags::R) {
        if current_flags.contains(PTEFlags::U) {
            p &= !((0b11 << 6) as usize);
            p |= (0b01 << 6) as usize;
        }
    }

    if f.contains(PTEFlags::X) {
        p &= !((3usize << 53) as usize);
        if current_flags.contains(PTEFlags::U) {
            p |= (0usize << 53) as usize
        } else {
            p |= (2usize << 53) as usize
        }
    }

    p
}

pub fn clear_flag(pte: usize, f: PTEFlags) -> usize {
    let mut p = pte;
    let current_flags = flags(pte).unwrap();

    if f.contains(PTEFlags::V) || f.contains(PTEFlags::R) {
        p &= !1usize;
    }

    if f.contains(PTEFlags::T) {
        p &= !(1usize << 1)
    }

    if f.contains(PTEFlags::W) {
        p &= !((3usize << 6) as usize);
        if current_flags.contains(PTEFlags::U) {
            p |= (1usize << 6) as usize;
        } else {
            p |= (3usize << 6) as usize;
        }
    }

    if f.contains(PTEFlags::X) {
        p &= !((3usize << 53) as usize);
        if current_flags.contains(PTEFlags::U) {
            p |= (2usize << 53) as usize;
        } else {
            p |= (3usize << 53) as usize;
        }
    }

    p
}