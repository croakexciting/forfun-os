#![allow(unused)]
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

register_bitfields! {
    u32,

    /// Distributor Control Register
    CTLR [
        Enable OFFSET(0) NUMBITS(1) []
    ],

    /// Interrupt Controller Type Register
    TYPER [
        ITLinesNumber OFFSET(0)  NUMBITS(5) []
    ],

    /// Interrupt Processor Targets Registers
    ITARGETSR [
        Offset3 OFFSET(24) NUMBITS(8) [],
        Offset2 OFFSET(16) NUMBITS(8) [],
        Offset1 OFFSET(8)  NUMBITS(8) [],
        Offset0 OFFSET(0)  NUMBITS(8) []
    ],

    /// Interrupt Priority Mask Register
    PMR [
        Priority OFFSET(0) NUMBITS(8) []
    ],

    /// Interrupt Acknowledge Register
    IAR [
        InterruptID OFFSET(0) NUMBITS(10) []
    ],

    /// End of Interrupt Register
    EOIR [
        EOIINTID OFFSET(0) NUMBITS(10) []
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    pub GICDPPIBlock {
        (0x000 => _reserved1),
        (0x100 => ISENABLER: ReadWrite<u32>),
        (0x104 => _reserved2),
        (0x800 => ITARGETSR: [ReadOnly<u32, ITARGETSR::Register>; 8]),
        (0x820 => @END),
    },

    #[allow(non_snake_case)]
    pub GICCBlock {
        (0x000 => CTLR: ReadWrite<u32, CTLR::Register>),
        (0x004 => PMR: ReadWrite<u32, PMR::Register>),
        (0x008 => _reserved1),
        (0x00C => IAR: ReadWrite<u32, IAR::Register>),
        (0x010 => EOIR: ReadWrite<u32, EOIR::Register>),
        (0x014  => @END),
    }
}

pub struct GICD {
    addr: usize,
}

impl GICD {
    pub fn new(addr: usize) -> Self {
        Self { addr }
    }

    pub fn enable(&self, irq_num: usize) {
        let enable_reg_index = irq_num >> 5;
        let enable_bit = 1u32 << (irq_num % 32);
        match irq_num {
            // PPI
            0..=31 => {
                let ppi = unsafe { &*(self.addr as *const GICDPPIBlock) };
                ppi.ISENABLER.set(ppi.ISENABLER.get() | enable_bit);
            }
            // SPI
            _ => {
                println!("[kernel] enable irq: {}", irq_num);
            }
        }
    }
}

pub struct GICC {
    addr: usize
}

impl GICC {
    pub fn new(addr: usize) -> Self {
        Self { addr }
    }

    pub fn enable(&self) {
        let gic = unsafe { &*(self.addr as *const GICCBlock) };
        gic.CTLR.write(CTLR::Enable::SET);
    }

    pub fn set_priority(&self, priority: u32) {
        let gic = unsafe { &*(self.addr as *const GICCBlock) };
        gic.PMR.write(PMR::Priority.val(priority));
    }

    pub fn claim(&self) -> usize {
        let gic = unsafe { &*(self.addr as *const GICCBlock) };
        gic.IAR.read(IAR::InterruptID) as usize
    }

    pub fn complete(&self, irq_num: u32) {
        let gic = unsafe { &*(self.addr as *const GICCBlock) };
        gic.EOIR.write(EOIR::EOIINTID.val(irq_num));
    }
}

pub struct GICV2 {
    gicd: GICD,
    gicc: GICC,
}

impl GICV2 {
    pub fn new(gicc_addr: usize, gicd_addr: usize) -> GICV2 {
        Self { gicd: GICD::new(gicd_addr), gicc: GICC::new(gicc_addr) }
    }

    pub fn enable(&self, irq_num: usize) {
        self.gicd.enable(irq_num);
        self.gicc.enable();
    }

    pub fn set_priority(&self, priority: u32) {
        self.gicc.set_priority(priority);
    }
}