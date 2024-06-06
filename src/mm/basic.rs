use bitflags::{bitflags, Flags};

const PA_VALID_WIDTH: usize = 56;
const VA_VALID_WIDTH: usize = 39;
const PPN_VALID_WIDTH: usize = 44;
const VPN_VALID_WIDTH: usize = 27;
const INPAGE_OFFSET_WIDTH: usize = 12;

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct VirtAddr(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct PhysPage(pub usize);

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct VirtPage(pub usize);

impl From<usize> for PhysAddr {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PA_VALID_WIDTH) -1))
    }
}

impl From<usize> for PhysPage {
    fn from(value: usize) -> Self {
        Self(value & ((1 << PPN_VALID_WIDTH) -1))
    }
}

impl From<PhysAddr> for usize {
    fn from(value: PhysAddr) -> Self {
        value.0
    }
}

impl From<PhysPage> for usize {
    fn from(value: PhysPage) -> Self {
        value.0
    }
}

impl From<usize> for VirtAddr {
    fn from(value: usize) -> Self {
        Self(value & ((1 << VA_VALID_WIDTH) -1))
    }
}

impl From<usize> for VirtPage {
    fn from(value: usize) -> Self {
        Self(value & ((1 << VPN_VALID_WIDTH) -1))
    }
}

impl From<VirtAddr> for usize {
    fn from(value: VirtAddr) -> Self {
        value.0
    }
}

impl From<VirtPage> for usize {
    fn from(value: VirtPage) -> Self {
        value.0
    }
}

impl PhysAddr {
    pub fn page_number(&self) -> PhysPage {
        PhysPage(self.0 >> INPAGE_OFFSET_WIDTH)
    }
}

impl VirtAddr {
    pub fn page_number(&self) -> VirtPage {
        VirtPage(self.0 >> INPAGE_OFFSET_WIDTH)
    }
}

impl From<PhysAddr> for PhysPage {
    fn from(value: PhysAddr) -> Self {
        value.page_number()
    }
}

impl From<VirtAddr> for VirtPage {
    fn from(value: VirtAddr) -> Self {
        value.page_number()
    }
}

bitflags! {
    pub struct PTEFlags: u8 {
        const V = 1 << 0;
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
        const G = 1 << 5;
        const A = 1 << 6;
        const D = 1 << 7;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PageTableEntry(pub usize);

impl PageTableEntry {
    pub fn new(ppn: PhysPage, flags: PTEFlags) -> Self {
        // 8bit flags + 2bit rsw
        Self(ppn.0 << 10 | flags.bits() as usize)
    }

    pub fn ppn(&self) -> PhysPage {
        (self.0 >> 10).into()
    }

    pub fn flags(&self) -> Option<PTEFlags> {
        PTEFlags::from_bits(self.0 as u8)
    }

    pub fn is_set(&self, bit: PTEFlags) -> bool {
        if let Some(p) = self.flags() {
            if p.contains(bit) {
                return true;
            }
        }
        return false;
    }
}