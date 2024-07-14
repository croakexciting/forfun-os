use alloc::sync::Arc;

use crate::{
    arch::riscv64::{
        copy_from_user_into_vector, 
        copy_vector_to_user, 
        str_from_user,
        copy_usize_with_user
    }, 
    process::{
        connect_server, 
        create_server, 
        recv_request, 
        reply_request, 
        request, 
        sem_open, 
        sem_raise, 
        sem_wait, 
        shm_open
    }
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

pub fn sys_create_server(name: *const i8) -> isize {
    let str = str_from_user(name);
    create_server(str)
}

pub fn sys_connect_server(name: *const i8) -> isize {
    let str = str_from_user(name);
    connect_server(str)
}

pub fn sys_request(coid: usize, req: *const u8, req_len: usize, resp: *mut u8) -> isize {
    let req_data = copy_from_user_into_vector(req, req_len);
    if let Some(resp_data) = request(coid, Arc::new(req_data)) {
        // 确保数据在内核堆中已经被丢弃释放
        let raw_vec = Arc::try_unwrap(resp_data).unwrap();
        copy_vector_to_user(raw_vec, resp) as isize
    } else {
        -1
    }
}

pub fn sys_recv_request(name: *const i8, req: *mut u8, req_len: *mut usize, timeout_ms: usize) -> isize {
    let str = str_from_user(name);
    if let Some(req_data) = recv_request(str, timeout_ms) {
        // 确保数据在内核堆中已经被丢弃释放
        let raw_vec = Arc::try_unwrap(req_data.1).unwrap();
        let len = copy_vector_to_user(raw_vec, req);
        copy_usize_with_user(len, req_len);
        req_data.0 as isize
    } else {
        -1
    }
}

pub fn sys_replay_request(rcvid: usize, resp: *const u8, resp_len: usize) -> isize {
    let resp_data = copy_from_user_into_vector(resp, resp_len);
    reply_request(rcvid, Arc::new(resp_data))
}