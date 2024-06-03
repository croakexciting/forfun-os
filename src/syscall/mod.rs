// use linux syscall id
const SYSCALL_WRITE: usize = 1;
const SYSCALL_EXIT: usize = 60;

mod file;
mod process;

use file::*;
use process::*;

pub fn syscall(id: usize, args: [usize; 3]) -> isize {
    match id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        _ => panic!("Unsupported syscall id: {}", id),
    }
}