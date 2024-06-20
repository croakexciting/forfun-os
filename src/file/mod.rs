pub mod inode;
pub mod stdio;

use crate::mm::area::UserBuffer;

pub trait File: Send + Sync {
    #[allow(unused)]
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    #[allow(unused)]
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
}