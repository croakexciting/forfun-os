use crate::process::start_next_app;

pub fn sys_exit(code: i32) -> ! {
    println!("[kernel] Application exited with code {}", code);
    start_next_app();
}