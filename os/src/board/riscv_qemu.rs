pub const CLOCK_FREQ: usize = 12500000;
pub const PLIC_ADDR: usize = 0xC00_0000;

use crate::driver::plic::*;

pub fn board_init() {
    let plic = PLIC::new(PLIC_ADDR);
    plic.set_threshold(0, Level::Supervisor, 0);
    plic.set_threshold(0, Level::Machine, 1);
    //irq nums: 5 keyboard, 8 block, 10 uart
    for interrupt_id in [5usize, 8, 10] {
        plic.enable(0, Level::Supervisor, interrupt_id);
        plic.set_priority(interrupt_id, 1);
    }

    unsafe {
        riscv::register::sie::set_sext();
    }
}

pub fn irq_handler() {
    let plic = PLIC::new(PLIC_ADDR);
    let interrupt_id = plic.claim(0, Level::Supervisor);
    match interrupt_id {
        5 => {println!("IRQ {}", interrupt_id);},
        _ => {println!("unsupported IRQ {}", interrupt_id);},
    }

    plic.complete(0, Level::Supervisor, interrupt_id);
}