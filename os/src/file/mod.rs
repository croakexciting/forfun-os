pub mod stdio;
pub mod nomalfile;
pub mod fs;

use core::fmt;

use crate::mm::area::UserBuffer;
use bitflags::bitflags;
use rcore_fs::vfs::FsError;

pub trait File: Send + Sync {
    #[allow(unused)]
    fn read(&self, buf: &mut UserBuffer) -> Result<usize, FileError>;
    fn write(&self, buf: &UserBuffer) -> Result<usize, FileError>;
    #[allow(unused)]
    fn size(&self) -> Result<usize, FileError>;
    #[allow(unused)]
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn lseek(&self, seek: usize) -> isize;
}

bitflags! {
    #[derive(Clone, Copy)]
    pub struct FilePermission: u8 {
        const X = 1 << 0;
        const W = 1 << 1;
        const R = 1 << 2;
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum FileError {
    FsError(FsError),
    #[allow(unused)]
    EOF(usize),
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileError::FsError(err) => err.fmt(f),
            FileError::EOF(size) => write!(f, "EOF, read {} bytes", size),
        }
    }
}

impl From<FsError> for FileError {
    fn from(value: FsError) -> Self {
        FileError::FsError(value)
    }
}