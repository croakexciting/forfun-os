// dma 区域主要是为了 virtio，暂定物理内存位置为 0x87000000 ~ 0x88000000 16M

use crate::utils::type_extern::RefCellWrap;
use crate::arch::memory::page::{
    PhysPage, PhysAddr, VirtAddr, VirtPage, PAGE_SIZE
};
use crate::board::peri::memory::{DMA_START_ADDR, DMA_END_ADDR};
use super::buddy::BuddyAllocator;
use lazy_static::*;

lazy_static! {
    pub static ref DMA_ALLOCATOR: RefCellWrap<BuddyAllocator> = unsafe {
        let start_ppn: PhysPage = PhysAddr::from(DMA_START_ADDR).into();
        let pn = (DMA_END_ADDR - DMA_START_ADDR) / PAGE_SIZE;

        RefCellWrap::new(BuddyAllocator::new(5, start_ppn.0.into(), pn))
    };
}

pub fn dma_alloc(pn: usize) -> Option<usize> {
    let start = DMA_ALLOCATOR.exclusive_access().alloc(pn)?;
    Some(VirtAddr::from(start).0)
}

pub fn dma_dealloc(addr: usize, pn: usize) {
    let vpn: VirtPage = VirtAddr::from(addr).into();
    DMA_ALLOCATOR.exclusive_access().dealloc(vpn, pn)
}