use crate::{board::console_getchar, mm::area::UserBuffer};
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

pub struct Stdin;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }

    fn writable(&self) -> bool {
        false
    }

    fn read(&self, buf: &mut UserBuffer) -> isize {
        let c = console_getchar();
        if c as u8 == 0 {
            return 0;
        } else {
            buf.buffer[0] = c as u8;
            return 1;
        }
    }

    fn write(&self, _buf: &UserBuffer) -> isize {
        panic!("[kernel] Stdin can't write")
    }

    fn lseek(&self, seek: usize) -> isize {
        seek as isize
    }
}