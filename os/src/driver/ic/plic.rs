#![allow(unused)]

pub struct PLIC {
    addr: usize
}

#[derive(Copy, Clone)]
pub enum Level {
    Machine = 0,
    Supervisor = 1,
}

impl PLIC {
    pub fn set_addr(&mut self, addr: usize) {
        self.addr = addr
    }

    fn priority_ptr(&self, interrupt_id: usize) -> *mut u32 {
        assert!(interrupt_id > 0 && interrupt_id <= 127);
        (self.addr + interrupt_id * 4) as *mut u32
    }

    fn context_id(hart_id: usize, level: Level) -> usize {
        hart_id * 2 + level as usize
    }

    fn enable_ptr(&self, hart_id: usize, level: Level, interrupt_id: usize) -> (*mut u32, usize) {
        let ctx_id = Self::context_id(hart_id, level);
        let (reg_id, reg_shift) = (interrupt_id / 32, interrupt_id % 32);
        (
            (self.addr + 0x2000 + 0x80 * ctx_id + 0x4 * reg_id) as *mut u32,
            reg_shift,
        )
    }

    fn threshold_ptr(&self, hart_id: usize, level: Level) -> *mut u32 {
        let ctx_id = Self::context_id(hart_id, level);
        (self.addr + 0x20_0000 + 0x1000 * ctx_id) as *mut u32
    }

    fn claim_complete_ptr(&self, hart_id: usize, level: Level) -> *mut u32 {
        let ctx_id = Self::context_id(hart_id, level);
        (self.addr + 0x20_0004 + 0x1000 * ctx_id) as *mut u32
    }

    pub fn new (addr: usize) -> Self {
        Self { addr }
    }

    pub fn set_priority(&self, interrupt_id: usize, priority: u32) {
        unsafe {
            self.priority_ptr(interrupt_id).write_volatile(priority);
        }
    }

    #[allow(unused)]
    pub fn get_priority(&self, interrupt_id: usize) -> u32 {
        unsafe { self.priority_ptr(interrupt_id).read_volatile() & 0x7 }
    }

    pub fn enable(&self, hart_id: usize, level: Level, interrupt_id: usize) {
        let (reg_ptr, shift) = self.enable_ptr(hart_id, level, interrupt_id);
        unsafe { reg_ptr.write_volatile(reg_ptr.read_volatile() | 1u32 << shift); }
    }

    #[allow(unused)]
    pub fn disable(&self, hart_id: usize, level: Level, interrupt_id: usize) {
        let (reg_ptr, shift) = self.enable_ptr(hart_id, level, interrupt_id);
        unsafe { reg_ptr.write_volatile(reg_ptr.read_volatile() & (!(1u32 << shift))); }
    }

    pub fn set_threshold(&self, hart_id: usize, level: Level, threshold: u32) {
        let ptr = self.threshold_ptr(hart_id, level);
        unsafe { ptr.write_volatile(threshold); }
    }

    #[allow(unused)]
    pub fn get_threshold(&self, hart_id: usize, level: Level) -> u32 {
        let ptr = self.threshold_ptr(hart_id, level);
        unsafe { ptr.read_volatile() & 0x7 }
    }

    pub fn claim(&self, hart_id: usize, level: Level) -> u32 {
        let ptr = self.claim_complete_ptr(hart_id, level);
        unsafe { ptr.read_volatile() }
    }

    pub fn complete(&self, hart_id: usize, level: Level, completion: u32) {
        let ptr = self.claim_complete_ptr(hart_id, level);
        unsafe { ptr.write_volatile(completion) }
    }
}