use crate::utils::bits::clear_bit;
use core::arch::asm;

pub fn create_ctx(entry: usize) -> [usize; 34] {
    let mut ctx: [usize; 34] = [0; 34];
    unsafe {
        let mut sstatus: usize;
        asm!("csrr {}, sstatus", out(reg) sstatus);

        // set privilege mode to user
        // sstatus privilege bit SPP at bit 8; 0 -> user mod, 1 -> supervisor mod
        // save the value in trap context, add will set sstatus in __restore fn
        sstatus = clear_bit(sstatus, 8);

        ctx[32] = sstatus;
        ctx[33] = entry;
        ctx
    }
}