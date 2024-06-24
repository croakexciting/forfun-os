use crate::process::{sem_open, sem_raise, sem_wait, shm_open};

pub fn sys_shm_open(id: usize, size: usize, permission: usize) -> isize {
    shm_open(id, size, permission)
}

pub fn sys_sem_open(id: usize) -> isize {
    sem_open(id)
}

pub fn sys_sem_wait(id: usize) -> isize {
    sem_wait(id)
}

pub fn sys_sem_raise(id: usize) -> isize {
    sem_raise(id)
}