/* 
    该区域预留给外设，我们将外设统一映射到 0x90000000~0xA0000000 管理
    外设区域是不需要实际内存的
*/

use lazy_static::*;
use crate::arch::memory::page::{
    PhysPage, PhysAddr, VirtAddr, VirtPage, PAGE_SIZE
};
use crate::utils::type_extern::RefCellWrap;
use super::buddy::BuddyAllocator;

const PERIPHERAL_START_ADDR: usize = 0x9000_0000;
const PERIPHERAL_END_ADDR: usize = 0xA000_0000;

lazy_static! {
    pub static ref PERIPHERAL_ALLOCATOR: RefCellWrap<BuddyAllocator> = unsafe {
        let start_ppn: PhysPage = PhysAddr::from(PERIPHERAL_START_ADDR).into();
        let pn = (PERIPHERAL_START_ADDR - PERIPHERAL_END_ADDR) / PAGE_SIZE;

        RefCellWrap::new(BuddyAllocator::new(5, start_ppn.0.into(), pn))
    };
}

pub fn peripheral_alloc(pn: usize) -> Option<usize> {
    let start = PERIPHERAL_ALLOCATOR.exclusive_access().alloc(pn)?;
    Some(VirtAddr::from(start).0)
}

pub fn peripheral_dealloc(addr: usize, pn: usize) {
    let vpn: VirtPage = VirtAddr::from(addr).into();
    PERIPHERAL_ALLOCATOR.exclusive_access().dealloc(vpn, pn)
}