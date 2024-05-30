use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}

impl TrapContext {
    pub fn new(entry: usize, sp: usize) -> Self {
        unsafe {
            let mut s = sstatus::read();
        }
    }

    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
}