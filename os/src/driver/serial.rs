// TODO: 增加设备树解析，配置驱动
use core::ptr;

use crate::board;

// use lazy_static::*;
// use crate::utils::type_extern::RefCellWrap;

// lazy_static! {
//     pub static ref CONSOLE: RefCellWrap<Uart> = unsafe {
//         RefCellWrap::new(Uart::new(0x1000_0000))
//     };
// }

pub struct Uart {
    addr: usize
}

impl Uart {
    pub fn new(addr: usize) -> Self {
        Self { addr }
    }

    pub fn put(&self, c: char) {
        unsafe {
            ptr::write_volatile(self.addr as *mut char, c);
        }
    }
}
