// use linux syscall id
const SYSCALL_WRITE: usize = 1;
const SYSCALL_YIELD: usize = 24;
const SYSCALL_NANOSLEEP: usize = 35;
const SYSCALL_EXIT: usize = 60;

mod file;
mod process;

use file::*;
use process::*;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    println!("syscall with id {}", id);
    match id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => {sys_yield(); 0},
        SYSCALL_NANOSLEEP => {sys_nanosleep(args[0] as usize); 0},
        _ => panic!("Unsupported syscall id: {}", id),
    }
}