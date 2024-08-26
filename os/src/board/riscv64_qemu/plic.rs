use crate::{driver::plic::qemu_plic::*, println, process};

pub const PLIC_ADDR: usize = 0xC00_0000;

pub fn plic_init() -> PLIC {
    let va = process::map_peripheral(PLIC_ADDR, 0x40_0000);
    let plic = PLIC::new(va as usize);
    plic.set_threshold(0, Level::Supervisor, 0);
    plic.set_threshold(0, Level::Machine, 1);
    plic.enable(0, Level::Supervisor, 10);
    plic.set_priority(10, 1);

    unsafe {
        riscv::register::sie::set_sext();
    }

    plic
}

// pub fn external_irq_handler() {
//     let irq_id = super::PLIC.exclusive_access().claim(0, Level::Supervisor);
//     match irq_id {
//         10 => {println!("IRQ {}", irq_id);},
//         _ => {println!("unsupported IRQ {}", irq_id);},
//     }

//     super::PLIC.exclusive_access().complete(0, Level::Supervisor, irq_id);
// }