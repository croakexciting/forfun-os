use crate::{driver::plic::qemu_plic::*, println};

pub const PLIC_ADDR: usize = 0xC00_0000;

pub fn plic_init() -> PLIC {
    let plic = PLIC::new(PLIC_ADDR);
    plic.set_threshold(0, Level::Supervisor, 0);
    plic.set_threshold(0, Level::Machine, 1);
    plic.enable(0, Level::Supervisor, 10);
    plic.set_priority(10, 1);

    unsafe {
        riscv::register::sie::set_sext();
    }

    plic
}

pub fn external_irq_handler() {
    let plic = super::PLIC.exclusive_access();
    let irq_id = plic.claim(0, Level::Supervisor);
    match irq_id {
        10 => {println!("IRQ {}", irq_id);},
        _ => {println!("unsupported IRQ {}", irq_id);},
    }

    plic.complete(0, Level::Supervisor, irq_id);
}