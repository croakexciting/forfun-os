use crate::process::*;
use crate::arch::riscv64::copy_usize_with_user;

/// write buf of length `len`  to a file with `fd`
/// TODO: only support stdout write, modify this after add filesystem
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    write(fd, buf as *mut u8, len)
}

pub fn sys_read(fd: usize, buf: *mut u8, len: usize) -> isize {
    read(fd, buf, len)
}

pub fn sys_create_pipe(buf: *mut usize) -> isize {
    unsafe {
        let arr = core::slice::from_raw_parts_mut(buf, 2);
        let (read_end, write_end) = create_pipe(4096);
        copy_usize_with_user(read_end, &mut arr[0]);
        copy_usize_with_user(write_end, &mut arr[1]);
        0
    }
} 