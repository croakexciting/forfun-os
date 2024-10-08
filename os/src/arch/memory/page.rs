use bitflags::bitflags;
use crate::arch::inner::memory::page;

const VA_VALID_WIDTH: usize = page::VA_VALID_WIDTH;
const PN_LEVEL_NUM: usize = page::PN_LEVEL_NUM;
const PN_BITSIZE: usize = page::PN_BITSIZE;
pub const INPAGE_OFFSET_WIDTH: usize = page::INPAGE_OFFSET_WIDTH;
pub const PAGE_SIZE: usize = page::PAGE_SIZE;

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
        Self(value)
    }
}

impl From<usize> for PhysPage {
    fn from(value: usize) -> Self {
        Self(value)
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
        Self(value)
    }
}

impl From<usize> for VirtPage {
    fn from(value: usize) -> Self {
        Self(value)
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

    pub fn is_kernel(&self) -> bool {
        if self.0 >> (VA_VALID_WIDTH - 1) > 0 {
            return true;
        } else {
            false
        }
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
        unsafe {core::slice::from_raw_parts_mut(addr.0 as *mut u8, PAGE_SIZE)}
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

/* 
    riscv 中页表等级从高到底叫做 2,1,0 级
    为了方便管理，我们还是采用 0,1,2 的方式, 将最高层页表 index 放在 [0]
*/
impl VirtPage {
    pub fn pte_array(&self) -> &'static mut [PageTableEntry] {
        let addr: VirtAddr = (*self).into();
        unsafe {core::slice::from_raw_parts_mut(addr.0 as *mut PageTableEntry, 512)}
    }

    pub fn index(&self) -> [usize; PN_LEVEL_NUM] {
        let mut vpn = self.0;
        let mut idx = [0usize; PN_LEVEL_NUM];
        for i in (0..PN_LEVEL_NUM).rev() {
            // 低9位有效
            idx[i] = vpn & ((1 << PN_BITSIZE) - 1);
            vpn >>= PN_BITSIZE;
        }
        idx
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

    pub fn bytes_array(&self) -> &'static mut [u8] {
        let addr: VirtAddr = (*self).into();
        unsafe {core::slice::from_raw_parts_mut(addr.0 as *mut u8, PAGE_SIZE)}
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
        // table or block
        const T = 1 << 5;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct PageTableEntry(pub usize);

impl PageTableEntry {
    pub fn new(ppn: PhysPage, flags: PTEFlags) -> Self {
        Self(page::pte(ppn.0, flags))
    }

    pub fn ppn(&self) -> PhysPage {
        (page::ppn(self.0)).into()
    }

    pub fn flags(&self) -> Option<PTEFlags> {
        page::flags(self.0)
    }

    pub fn is_valid(&self) -> bool {
        page::is_set(self.0, PTEFlags::V)
    }

    pub fn is_set(&self, flag: PTEFlags) -> bool {
        page::is_set(self.0, flag)
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }

    pub fn clear_flag(&mut self, bit: PTEFlags) {
        self.0 = page::clear_flag(self.0, bit)
    }

    pub fn set_flag(&mut self, bit: PTEFlags) {
        self.0 = page::set_flag(self.0, bit)
    }
}

pub fn root_ppn() -> usize {
    crate::arch::inner::memory::page::root_ppn()
}

pub fn enable_va(id: usize, ppn: usize) {
    crate::arch::inner::memory::page::enable_va(id, ppn)
}

pub fn flush_tlb(asid: usize) {
    unsafe {
        crate::arch::inner::memory::page::flush_tlb(asid);
    }
}

pub fn kernel_phys_to_virt(phys: PhysAddr) -> VirtAddr {
    (0xFFFF_FFFF_0000_0000usize + phys.0).into()
}

pub fn kernel_virt_to_phys(virt: VirtAddr) -> PhysAddr {
    (virt.0 - 0xFFFF_FFFF_0000_0000usize).into()
}

pub fn kernel_page_phys_to_virt(phys: PhysPage) -> VirtPage {
    (0xFFFF_FFFF_0000_0usize + phys.0).into()
}

#[allow(unused)]
pub fn kernel_page_virt_to_phys(virt: VirtPage) -> PhysPage {
    (virt.0 - 0xFFFF_FFFF_0000_0usize).into()
}