pub mod pt;
pub mod basic;
pub mod allocator;
pub mod area;

use alloc::vec;
use alloc::vec::Vec;
use area::{MapArea, Permission, MapType};
use pt::PageTable;

// 出于简单考虑，我第一步计划是将整个内存空间算作一个 map aera
const KERNEL_START_ADDR: usize = 0x80020000;
const KERNEL_END_ADDR: usize = 0x80200000;
// The memory manager for a process
pub struct MemoryManager {
    pt: PageTable,
    areas: Vec<MapArea>,
}

impl MemoryManager {
    pub fn new() -> Self {
        let mut pt = PageTable::new();

        // default map all kernel space
        let mut kernel_area = MapArea::new(
            KERNEL_START_ADDR.into(), 
            KERNEL_END_ADDR.into(), 
            MapType::Identical, 
            Permission::R | Permission::W | Permission::X
        );

        kernel_area.map(&mut pt);

        Self { pt, areas: vec![kernel_area] }
    }
}
