#![allow(unused)]
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};
use ITARGETSR::Offset0;

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
    pub GICDBlock {
        (0x000 => CTLR: ReadWrite<u32, CTLR::Register>),
        (0x004 => TYPER: ReadOnly<u32, TYPER::Register>),
        (0x008 => _reserved1),
        (0x100 => ISENABLER: [ReadWrite<u32>; 32]),
        (0x180 => _reserved2),
        (0x800 => ITARGETSR: [ReadWrite<u32, ITARGETSR::Register>; 256]),
        (0xC00 => @END),
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

    pub fn set_addr(&mut self, addr: usize) {
        self.addr = addr
    }

    pub fn enable(&self, irq_num: usize) {
        let enable_reg_index = irq_num >> 5;
        let enable_bit = 1u32 << (irq_num % 32);
        let gicd = unsafe { &*(self.addr as *const GICDBlock) };
        gicd.CTLR.write(CTLR::Enable::SET);
        gicd.ISENABLER[enable_reg_index].set(gicd.ISENABLER[enable_reg_index].get() | enable_bit);
        gicd.ITARGETSR[irq_num / 4].write(ITARGETSR::Offset1.val(0xFF) + ITARGETSR::Offset0.val(0xFF) + ITARGETSR::Offset2.val(0xFF) + ITARGETSR::Offset3.val(0xFF));
    }
}

pub struct GICC {
    addr: usize
}

impl GICC {
    pub fn new(addr: usize) -> Self {
        Self { addr }
    }

    pub fn set_addr(&mut self, addr: usize) {
        self.addr = addr
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
    pub fn new(addr: usize) -> GICV2 {
        Self { gicd: GICD::new(addr), gicc: GICC::new(addr + 0x10000) }
    }

    pub fn set_addr(&mut self, addr: usize) {
        self.gicc.set_addr(addr + 0x10000);
        self.gicd.set_addr(addr);
    }

    pub fn enable(&self, irq_num: usize) {
        self.gicd.enable(irq_num);
        self.gicc.enable();
    }

    pub fn set_priority(&self, priority: u32) {
        self.gicc.set_priority(priority);
    }

    pub fn claim(&self) -> usize {
        self.gicc.claim()
    }

    pub fn complete(&self, irq_num: usize) {
        self.gicc.complete(irq_num as u32);
    }
}