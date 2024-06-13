use crate::arch::riscv64::copy_from_user_into_vector;

const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
/// TODO: only support stdout write, modify this after add filesystem
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let data = copy_from_user_into_vector(buf, len);
            let str = core::str::from_utf8(data.as_slice()).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}