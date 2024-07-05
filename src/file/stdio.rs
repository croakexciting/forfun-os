use crate::mm::area::UserBuffer;
use super::File;
pub struct Stdout;

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        true
    }

    fn read(&self, _buf: &mut UserBuffer) -> isize {
        panic!("Cannot read from stdout")
    }

    fn write(&self, user_buf: &UserBuffer) -> isize {
        let data = user_buf.copy_to_vector();
        let str = core::str::from_utf8(data.as_slice()).unwrap();
        print!("{}", str);
        data.len() as isize
    }

    fn lseek(&self, offset: usize) -> isize {
        offset as isize
    }
}