// TODO: 增加设备树解析，配置驱动
use core::ptr;

pub struct Uart {
    addr: usize
}

impl Uart {
    pub fn new(addr: usize) -> Self {
        Self { addr }
    }

    pub fn set_addr(&mut self, addr: usize) {
        self.addr = addr
    }

    pub fn put(&self, c: char) {
        unsafe {
            ptr::write_volatile(self.addr as *mut char, c);
        }
    }
}
