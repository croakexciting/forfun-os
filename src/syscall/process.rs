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

pub fn sys_exec(addr: usize) -> isize {
    exec(addr)
}

pub fn sys_wait(pid: isize) -> isize {
    wait(pid)
}

pub fn sys_sigaction(signal: usize, handler: usize) -> isize {
    sigaction(signal, handler)
}

pub fn sys_set_signalmask(signal: usize) -> isize {
    set_signalmask(signal)
}

pub fn sys_sigreturn() -> isize {
    sigreturn();
    0
}

pub fn sys_getpid() -> isize {
    getpid() as isize
}

pub fn sys_kill(pid: usize, signal: usize) -> isize {
    set_signal(Some(pid), signal)
}