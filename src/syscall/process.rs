use crate::process::*;

pub fn sys_exit(code: isize) -> ! {
    exit(code as isize);
}

pub fn sys_yield() {
    sleep(0);
}

pub fn sys_nanosleep(duration: usize) {
    sleep(duration)
}

pub fn sys_fork() -> isize {
    // clone current process and create a new process
    // 如果不执行 exec 的话，子进程与父进程完全相同，并会继续执行下去
    fork()
}
