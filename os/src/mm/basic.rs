use bitflags::bitflags;

const PA_VALID_WIDTH: usize = 56;
const VA_VALID_WIDTH: usize = 39;
const PPN_VALID_WIDTH: usize = 44;
const VPN_VALID_WIDTH: usize = 27;
pub const INPAGE_OFFSET_WIDTH: usize = 12;
pub const PAGE_SIZE: usize = 0x1000;

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

    pub fn add(&self, i: usize) -> Self {
        Self(self.0 + i)
    }

    pub fn reduce(&self, i: usize) -> Self {
        Self(self.0 - i)
    }
}

impl VirtAddr {
    pub fn page_number(&self) -> VirtPage {
        VirtPage(self.0 >> INPAGE_OFFSET_WIDTH)
    }

    pub fn add(&self, size: usize) -> Self {
        VirtAddr(self.0 + size)
    }

    pub fn reduce(&self, i: usize) -> Self {
        Self(self.0 - i)
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

impl From<PhysPage> for PhysAddr {
    fn from(value: PhysPage) -> Self {
        PhysAddr(value.0 << INPAGE_OFFSET_WIDTH)
    }
}

impl From<VirtPage> for VirtAddr {
    fn from(value: VirtPage) -> Self {
        VirtAddr(value.0 << INPAGE_OFFSET_WIDTH)
    }
}

impl PhysPage {
    pub fn pte_array(&self) -> &'static mut [PageTableEntry] {
        let addr: PhysAddr = (*self).into();
        unsafe {core::slice::from_raw_parts_mut(addr.0 as *mut PageTableEntry, 512)}
    }

    pub fn bytes_array(&self) -> &'static mut [u8] {
        let addr: PhysAddr = (*self).into();
        unsafe {core::slice::from_raw_parts_mut(addr.0 as *mut u8, 4096)}
    }

    pub fn clear_page(&self) {
        let slice = self.bytes_array();
        slice.fill(0);
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn prev(&self) -> Self {
        Self(self.0 - 1)
    }

    pub fn add(&self, size: usize) -> Self {
        Self(self.0 + size)
    }
}

// riscv 中页表等级从高到底叫做 2,1,0 级
// 这种命名方式和 X86 是刚好相反的，但是命名我们无需关心，
// 为了方便管理，我们还是采用 0,1,2 的方式, 将最高层页表 index 放在 [0]
impl VirtPage {
    pub fn index(&self) -> [usize; 3] {
        let mut vpn = self.0;
        let mut idx = [0usize; 3];
        for i in (0..3).rev() {
            // 低9位有效
            idx[i] = vpn & 511;
            vpn >>= 9;
        }
        idx
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn prev(&self) -> Self {
        Self(self.0 - 1)
    }

    pub fn add(&self, size: usize) -> Self {
        Self(self.0 + size)
    }

    pub fn bytes_array(&self) -> &'static mut [u8] {
        let addr: VirtAddr = (*self).into();
        unsafe {core::slice::from_raw_parts_mut(addr.0 as *mut u8, 4096)}
    }
}

bitflags! {
    #[derive(Clone, Copy)]
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

    pub fn is_valid(&self) -> bool {
        self.is_set(PTEFlags::V)
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }

    pub fn clear_flag(&mut self, bit: PTEFlags) {
        if let Some(mut p) = self.flags() {
            p.remove(bit);
            let mask = 0xFFFFFFFFFFFFFF00 | p.bits() as usize;
            self.0 &= mask;
        }
    }

    pub fn set_flag(&mut self, bit: PTEFlags) {
        if let Some(mut p) = self.flags() {
            p.insert(bit);
            self.0 |= p.bits() as usize;
        }
    }
}