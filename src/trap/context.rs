use crate::utils::bits::clear_bit;
use core::arch::asm;

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32],
    pub sstatus: usize,
    pub sepc: usize,
}

impl TrapContext {
    pub fn new(entry: usize) -> Self {
        unsafe {
            // read sstatus
            let mut sstatus: usize;
            asm!("csrr {}, sstatus", out(reg) sstatus);
            
            // set privilege mode to user
            // sstatus privilege bit SPP at bit 8; 0 -> user mod, 1 -> supervisor mod
            // save the value in trap context, add will set sstatus in __restore fn
            sstatus = clear_bit(sstatus, 8);

            TrapContext {
                x: [0; 32],
                sstatus,
                sepc: entry,
            }
        }
    }
}