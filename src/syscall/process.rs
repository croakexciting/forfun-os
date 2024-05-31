pub fn sys_exit(code: i32) -> ! {
    println!("[kernel] Application exited with code {}", code);
}