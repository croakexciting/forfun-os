use crate::process::{mmap, shm_open};

pub fn sys_mmap(size: usize, permission: usize) -> isize {
    mmap(size, permission)
}

pub fn sys_shm_open(id: usize, size: usize, permission: usize) -> isize {
    shm_open(id, size, permission)
}