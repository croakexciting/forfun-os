use core::arch::asm;
use core::ffi::CStr;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use crate::mm::basic::VirtPage;

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

pub fn copy_from_user_into_vector(from: *const u8, n: usize) -> Vec<u8> {
    let mut vec = Vec::new();
    vec.resize(n, 0);
    unsafe { copy_with_user(vec.as_mut_ptr(), from, n) }
    vec
}

pub fn copy_vector_to_user(v: Vec<u8>, dst: *mut u8) -> usize {
    unsafe { copy_with_user(dst, v.as_ptr(), v.len()) }
    v.len()
}

pub fn copy_user_page_to_vector(vpn: VirtPage) -> Vec<u8> {
    let src = vpn.bytes_array().as_ptr();
    copy_from_user_into_vector(src, 4096)
}

pub fn copy_vector_to_user_page(v: Vec<u8>, vpn: VirtPage) {
    let dst = vpn.bytes_array().as_mut_ptr();
    copy_vector_to_user(v, dst);
}

pub fn copy_usize_with_user(src: usize, dst: *mut usize) {
    unsafe {
        riscv::register::sstatus::set_sum();
        *dst = src;
        riscv::register::sstatus::clear_sum();
    }
}

pub fn str_from_user(src: *const i8) -> String {
    unsafe {
        riscv::register::sstatus::set_sum();
        let str = CStr::from_ptr(src).to_str().unwrap().to_string().clone();
        riscv::register::sstatus::clear_sum();
        str
    }
}
