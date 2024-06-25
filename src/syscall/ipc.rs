use crate::{
    arch::riscv64::str_from_user, 
    process::{sem_open, sem_raise, sem_wait, shm_open}
};

pub fn sys_shm_open(name: *const i8, size: usize, permission: usize) -> isize {
    let str = str_from_user(name);
    shm_open(str, size, permission)
}

pub fn sys_sem_open(name: *const i8) -> isize {
    let str = str_from_user(name);
    sem_open(str)
}

pub fn sys_sem_wait(name: *const i8) -> isize {
    let str = str_from_user(name);
    sem_wait(str)
}

pub fn sys_sem_raise(name: *const i8) -> isize {
    let str = str_from_user(name);
    sem_raise(str)
}