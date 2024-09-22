#![allow(unused)]

// use linux syscall id
const SYSCALL_READ: usize = 0;
const SYSCALL_WRITE: usize = 1;
const SYSCALL_OPEN: usize = 2;
const SYSCALL_CLOSE: usize = 3;
const SYSCALL_LSEEK: usize = 4;
const SYSCALL_SIZE: usize = 5;
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

mod file;
mod process;
mod mm;
mod ipc;

use file::*;
use process::*;
use mm::*;
use ipc::*;

pub fn syscall(id: usize, args: [usize; 4]) -> isize {
    match id {
        SYSCALL_READ => sys_read(args[0], args[1] as *mut u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_OPEN => sys_open(args[0] as *const i8),
        SYSCALL_LSEEK => sys_lseek(args[0], args[1]),
        SYSCALL_SIZE => sys_size(args[0]),
        SYSCALL_MMAP => sys_mmap(args[0] as usize, args[1] as usize),
        SYSCALL_UMMAP => sys_ummap(args[0]),
        SYSCALL_MMAP_WITH_ADDR => sys_mmap_with_addr(args[0], args[1], args[2]),
        SYSCALL_SIG => sys_set_signal(args[0], args[1]),
        SYSCALL_SIGACTION => sys_sigaction(args[0], args[1]),
        SYSCALL_SIGPROCMASK => sys_set_signalmask(args[0]),
        SYSCALL_SIGRETURN => sys_sigreturn(),
        SYSCALL_EXIT => sys_exit(args[0] as isize),
        SYSCALL_YIELD => {sys_yield(); 0},
        SYSCALL_NANOSLEEP => {sys_nanosleep(args[0] as usize); 0},
        SYSCALL_FORK => {sys_fork()},
        SYSCALL_EXEC => {sys_exec(args[0] as *mut u8, args[1])},
        SYSCALL_WAIT => {sys_wait(args[0] as isize)},
        SYSCALL_PIPE => sys_create_pipe(args[0] as *mut usize),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_KILL => sys_kill(args[0] as usize, args[1] as usize),
        SYSCALL_SHM_OPEN => sys_shm_open(args[0] as *const i8, args[1], args[2]),
        SYSCALL_SEM_OPEN => sys_sem_open(args[0] as *const i8),
        SYSCALL_SEM_WAIT => sys_sem_wait(args[0] as *const i8),
        SYSCALL_SEM_RAISE => sys_sem_raise(args[0] as *const i8),
        SYSCALL_SRV_CREATE => sys_create_server(args[0] as *const i8),
        SYSCALL_SRV_CONNECT => sys_connect_server(args[0] as *const i8),
        SYSCALL_SRV_REQUEST => sys_request(args[0], args[1] as *const u8, args[2], args[3] as *mut u8),
        SYSCALL_SRV_RECV => sys_recv_request(args[0] as *const i8, args[1] as *mut u8, args[2] as *mut usize, args[3]),
        SYSCALL_SRV_REPLY => sys_replay_request(args[0], args[1] as *const u8, args[2]),
        _ => panic!("Unsupported syscall id: {}", id),
    }
}