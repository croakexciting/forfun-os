use aarch64_cpu::{asm::barrier, registers::*};
use crate::arch::context::TrapContext;
use core::arch::global_asm;

global_asm!(include_str!("trap.S"));

pub fn init() {
    extern "C" {
        fn __alltraps();
    }

    VBAR_EL1.set(__alltraps as u64);
    barrier::isb(barrier::SY);
}

#[no_mangle]
pub fn current_elx_synchronous(ctx: &mut TrapContext) {
    
}

#[no_mangle]
pub fn current_elx_irq(ctx: &mut TrapContext) {

}

#[no_mangle]
pub fn current_elx_serror(ctx: &mut TrapContext) {

}

#[no_mangle]
pub fn lower_aarch64_synchronous(ctx: &mut TrapContext) {

}

#[no_mangle]
pub fn lower_aarch64_irq(ctx: &mut TrapContext) {

}

#[no_mangle]
pub fn lower_aarch64_serror(ctx: &mut TrapContext) {
    
}