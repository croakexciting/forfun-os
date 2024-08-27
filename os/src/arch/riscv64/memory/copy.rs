use core::arch::asm;
use core::ffi::CStr;

use alloc::string::{String, ToString};

#[inline]
pub unsafe fn copy_with_user(to: *mut u8, from: *const u8, n: usize) {
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

pub fn copy_usize_with_user(src: usize, dst: *mut usize) {
    unsafe {
        riscv::register::sstatus::set_sum();
        *dst = src;
        riscv::register::sstatus::clear_sum();
    }
}

pub fn copy_str_with_user(src: *const i8) -> String {
    unsafe {
        riscv::register::sstatus::set_sum();
        let str = CStr::from_ptr(src).to_str().unwrap().to_string().clone();
        riscv::register::sstatus::clear_sum();
        str
    }
}

pub fn enable_user_access() {
    unsafe {riscv::register::sstatus::set_sum()}
}

pub fn disable_user_access() {
    unsafe {riscv::register::sstatus::clear_sum()}
}
