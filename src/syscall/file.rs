use crate::process::write;

/// write buf of length `len`  to a file with `fd`
/// TODO: only support stdout write, modify this after add filesystem
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    write(fd, buf as *mut u8, len)
}