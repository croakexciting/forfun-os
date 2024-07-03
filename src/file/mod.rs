pub mod stdio;

use crate::mm::area::UserBuffer;

pub trait File: Send + Sync {
    #[allow(unused)]
    fn read(&self, buf: &mut UserBuffer) -> isize;
    fn write(&self, buf: &UserBuffer) -> isize;
    #[allow(unused)]
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
}