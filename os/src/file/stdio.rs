use crate::{board::console_getchar, mm::area::UserBuffer};
use super::{File, FileError};

pub struct Stdout;

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }

    fn writable(&self) -> bool {
        true
    }

    fn read(&self, _buf: &mut UserBuffer) -> Result<usize, FileError> {
        panic!("Cannot read from stdout")
    }

    fn write(&self, user_buf: &UserBuffer) -> Result<usize, FileError> {
        let data = user_buf.copy_to_vector();
        let str = core::str::from_utf8(data.as_slice()).unwrap();
        print!("{}", str);
        Ok(data.len())
    }

    fn lseek(&self, offset: usize) -> isize {
        offset as isize
    }

    fn size(&self) -> Result<usize, FileError> {
        Ok(0)
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

    fn read(&self, buf: &mut UserBuffer) -> Result<usize, FileError> {
        let c = console_getchar();
        if c as u8 == 0 {
            return Ok(0);
        } else {
            buf.buffer[0] = c as u8;
            return Ok(1);
        }
    }

    fn write(&self, _buf: &UserBuffer) -> Result<usize, FileError> {
        panic!("[kernel] Stdin can't write")
    }

    fn lseek(&self, seek: usize) -> isize {
        seek as isize
    }

    fn size(&self) -> Result<usize, FileError> {
        Ok(0)
    }
}