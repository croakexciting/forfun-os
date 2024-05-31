use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
}

impl TrapContext {
    pub fn new(entry: usize) -> Self {
        let s = sstatus::read();
        TrapContext {
            x: [0; 32],
            sstatus: s,
            sepc: entry,
        }
    }

    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
}