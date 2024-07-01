use crate::process::{mmap, mmap_with_addr, ummap};

pub fn sys_mmap(size: usize, permission: usize) -> isize {
    mmap(size, permission)
}

pub fn sys_ummap(addr: usize) -> isize {
    ummap(addr)
}

pub fn sys_mmap_with_addr(pa: usize, size: usize, permission: usize) -> isize {
    mmap_with_addr(pa, size, permission)
}