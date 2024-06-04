use crate::process::exit;

pub fn sys_exit(code: i32) -> ! {
    exit(code);
}