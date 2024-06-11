
const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
/// TODO: only support stdout write, modify this after add filesystem
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            println!("debug0");
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            println!("debug1 {:#x}", slice.as_ptr() as usize);
            let str = core::str::from_utf8(slice).unwrap();
            println!("debug2");
            print!("{}", str);
            println!("debug3");
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}