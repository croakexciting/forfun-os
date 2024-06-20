// use linux syscall id
const SYSCALL_READ: usize = 0;
const SYSCALL_WRITE: usize = 1;
const SYSCALL_PIPE: usize = 22;
const SYSCALL_YIELD: usize = 24;
const SYSCALL_NANOSLEEP: usize = 35;
const SYSCALL_EXIT: usize = 60;
const SYSCALL_FORK: usize = 57;
const SYSCALL_EXEC: usize = 59;
const SYSCALL_WAIT: usize = 61;

mod file;
mod process;

use file::*;
use process::*;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYSCALL_READ => sys_read(args[0], args[1] as *mut u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as isize),
        SYSCALL_YIELD => {sys_yield(); 0},
        SYSCALL_NANOSLEEP => {sys_nanosleep(args[0] as usize); 0},
        SYSCALL_FORK => {sys_fork()},
        SYSCALL_EXEC => {sys_exec(args[0] as usize)},
        SYSCALL_WAIT => {sys_wait(args[0] as isize)},
        SYSCALL_PIPE => sys_create_pipe(args[0] as *mut usize),
        _ => panic!("Unsupported syscall id: {}", id),
    }
}