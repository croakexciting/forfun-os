#![allow(unused)]
#![allow(dead_code)]

use core::{arch::asm, usize};

// use linux syscall id
const SYSCALL_READ: usize = 0;
const SYSCALL_WRITE: usize = 1;
const SYSCALL_OPEN: usize = 2;
const SYSCALL_CLOSE: usize = 3;
const SYSCALL_LSEEK: usize = 4;
const SYSCALL_MMAP: usize = 9;
const SYSCALL_UMMAP: usize = 10;
const SYSCALL_MMAP_WITH_ADDR: usize = 11;
const SYSCALL_SIG: usize = 12;
const SYSCALL_SIGACTION: usize = 13;
const SYSCALL_SIGPROCMASK: usize = 14;
const SYSCALL_SIGRETURN: usize = 15;
const SYSCALL_PIPE: usize = 22;
const SYSCALL_YIELD: usize = 24;
const SYSCALL_NANOSLEEP: usize = 35;
const SYSCALL_GETPID: usize = 39;
const SYSCALL_FORK: usize = 57;
const SYSCALL_EXEC: usize = 59;
const SYSCALL_EXIT: usize = 60;
const SYSCALL_WAIT: usize = 61;
const SYSCALL_KILL: usize = 62;
const SYSCALL_SHM_OPEN: usize = 70;
const SYSCALL_SEM_OPEN: usize = 80;
const SYSCALL_SEM_WAIT: usize = 81;
const SYSCALL_SEM_RAISE: usize = 82;
const SYSCALL_SRV_CREATE: usize = 90;
const SYSCALL_SRV_CONNECT: usize = 91;
const SYSCALL_SRV_REQUEST: usize = 92;
const SYSCALL_SRV_RECV: usize = 93;
const SYSCALL_SRV_REPLY: usize = 94;

fn syscall(id: usize, args: [usize; 4]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x13") args[3],
            in("x17") id
        );
    }
    ret
}

pub fn sys_read(fd: usize, buffer: &mut [u8]) -> isize {
    syscall(SYSCALL_READ, [fd, buffer.as_ptr() as usize, buffer.len(), 0])
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len(), 0])
}

pub fn sys_open(name: &str) -> isize {
    syscall(SYSCALL_OPEN, [name.as_ptr() as usize, 0, 0, 0])
}

pub fn sys_lseek(fd: usize, seek: usize) -> isize {
    syscall(SYSCALL_LSEEK, [fd, seek, 0, 0])
}

pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0, 0])
}

pub fn sys_yield() {
    syscall(SYSCALL_YIELD, [0, 0, 0, 0]);
}

pub fn sys_nanosleep(duration: usize) {
    syscall(SYSCALL_NANOSLEEP, [duration, 0, 0, 0]);
}

pub fn sys_fork() -> isize {
    syscall(SYSCALL_FORK, [0, 0, 0, 0])
}

pub fn sys_exec(elf: &[u8]) -> isize {
    syscall(SYSCALL_EXEC, [elf.as_ptr() as usize, elf.len(), 0, 0])
}

pub fn sys_wait(pid: usize) -> isize {
    syscall(SYSCALL_WAIT, [pid, 0, 0, 0])
}

pub fn sys_create_pipe(fd: &mut [usize]) -> isize {
    syscall(SYSCALL_PIPE, [fd.as_mut_ptr() as usize, 0, 0, 0])
}

pub fn sys_sig(pid: usize, signal: usize) -> isize {
    syscall(SYSCALL_SIG, [pid, signal, 0, 0])
}

pub fn sys_sigaction(signal: usize, handler: usize) -> isize {
    syscall(SYSCALL_SIGACTION, [signal, handler, 0, 0])
}

pub fn sys_sigprocmask(mask: usize) -> isize {
    syscall(SYSCALL_SIGPROCMASK, [mask, 0, 0, 0])
}

pub fn sys_sigreturn() -> isize {
    syscall(SYSCALL_SIGRETURN, [0, 0, 0, 0])
}

pub fn sys_getpid() -> isize {
    syscall(SYSCALL_GETPID, [0, 0, 0, 0])
}

pub fn sys_kill(pid: usize, signal: usize) -> isize {
    syscall(SYSCALL_KILL, [pid, signal, 0, 0])
}

pub fn sys_mmap(size: usize, permission: usize) -> isize {
    syscall(SYSCALL_MMAP, [size, permission, 0, 0])
}

pub fn sys_ummap(addr: usize) -> isize {
    syscall(SYSCALL_UMMAP, [addr, 0, 0, 0])
}

pub fn sys_shm_open(name: &str, size: usize, permission: usize) -> isize {
    syscall(SYSCALL_SHM_OPEN, [name.as_ptr() as usize, size, permission, 0])
}

pub fn sys_sem_open(name: &str) -> isize {
    syscall(SYSCALL_SEM_OPEN, [name.as_ptr() as usize, 0, 0, 0])
}

pub fn sys_sem_wait(name: &str) -> isize {
    syscall(SYSCALL_SEM_WAIT, [name.as_ptr() as usize, 0, 0, 0])
}

pub fn sys_sem_raise(name: &str) -> isize {
    syscall(SYSCALL_SEM_RAISE, [name.as_ptr() as usize, 0, 0, 0])
}

pub fn sys_create_server(name: &str) -> isize {
    syscall(SYSCALL_SRV_CREATE, [name.as_ptr() as usize, 0, 0, 0])
}

pub fn sys_connect_server(name: &str) -> isize {
    syscall(SYSCALL_SRV_CONNECT, [name.as_ptr() as usize, 0, 0, 0])
}

pub fn sys_request_server(coid: usize, req: &[u8], resp: *mut u8) -> isize {
    syscall(SYSCALL_SRV_REQUEST, [coid, req.as_ptr() as usize, req.len(), resp as usize])
}

pub fn sys_recv_server(name: &str, req: *mut u8, req_len: *mut usize, timeout_ms: usize) -> isize {
    syscall(SYSCALL_SRV_RECV, [name.as_ptr() as usize, req as usize, req_len as usize, timeout_ms])
}

pub fn sys_reply_server(rcvid: usize, resp: &[u8]) -> isize {
    syscall(SYSCALL_SRV_REPLY, [rcvid, resp.as_ptr() as usize, resp.len(), 0])
}