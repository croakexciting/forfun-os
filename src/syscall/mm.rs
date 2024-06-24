use crate::process::mmap;

pub fn sys_mmap(size: usize, permission: usize) -> isize {
    mmap(size, permission)
}