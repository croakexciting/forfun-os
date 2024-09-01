use core::arch::asm;

pub fn create_ctx(entry: usize) -> [usize; 34] {
    let mut ctx: [usize; 34] = [0; 34];
    unsafe {
        let mut spsr: usize;
        asm!("mrs {0}, SPSR_EL1", out(reg) spsr);

        ctx[32] = spsr;
        ctx[33] = entry;
        ctx
    }
}