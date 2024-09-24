#![allow(unused)]

pub struct GICD {
    addr: usize
}

impl GICD {
    pub fn new(addr: usize) -> Self {

        Self { addr }
    }

    pub fn set_addr(&mut self, addr: usize) {
        self.addr = addr
    }

    // pub fn global_enable(&mut self)
}

pub struct GICC {

}

impl GICC {
    pub fn new() {}
}