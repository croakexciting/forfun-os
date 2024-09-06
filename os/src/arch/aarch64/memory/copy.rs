use core::{arch::asm, ffi::CStr};

use alloc::string::{String, ToString};

#[inline]
pub unsafe fn copy_with_user(to: *mut u8, from: *const u8, n: usize) {
    let mut remaining = n;
    let mut src = from;
    let mut dst = to;

    while remaining > 0 {
        let _byte: u8;
        asm!(
            "1: cmp {remaining}, #0",
            "beq 2",
            "ldrb w3, [{src}], #1",
            "strb w3, [{dst}], #1", 
            "subs {remaining}, {remaining}, #1",
            "bne 1",
            "2:",
            src = inout(reg) src,
            dst = inout(reg) dst,
            remaining = inout(reg) remaining,
            options(nostack, preserves_flags),
        )
    }
}

pub fn copy_usize_with_user(src: usize, dst: *mut usize) {
    unsafe {
        *dst = src;
    }
}

pub fn copy_str_with_user(src: *const i8) -> String {
    unsafe {
        let str = CStr::from_ptr(src).to_str().unwrap().to_string().clone();
        str
    }
}

pub fn enable_user_access() {
    // do nothing
}

pub fn disable_user_access() {
    // do nothing
}