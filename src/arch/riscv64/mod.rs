use core::arch::asm;

use alloc::vec::Vec;

#[inline]
pub unsafe fn copy_from_user(to: *mut u8, from: *const u8, n: usize) {
    // TODO: access check

    let mut remaining = n;
    let mut src = from;
    let mut dst = to;

    // enable SUM flag
    riscv::register::sstatus::set_sum();

    while remaining > 0 {
        let _byte: u8;
        asm!(
            "sfence.vma",
            "1: lb {byte}, 0({src})",
            "sb {byte}, 0({dst})",
            "addi {src}, {src}, 1",
            "addi {dst}, {dst}, 1",
            "addi {remaining}, {remaining}, -1",
            "bnez {remaining}, 1b",
            byte = out(reg) _byte,
            src = inout(reg) src,
            dst = inout(reg) dst,
            remaining = inout(reg) remaining,
            options(nostack, preserves_flags)
        );
    }

    riscv::register::sstatus::clear_sum();
}

pub fn copy_from_user_into_vector(from: *const u8, n: usize) -> Vec<u8>{
    let mut vec = Vec::new();
    vec.resize(n, 0);
    unsafe { copy_from_user(vec.as_mut_ptr(), from, n) }
    vec
}