pub mod pt;
pub mod basic;
pub mod allocator;
pub mod area;
pub mod elf;

use alloc::vec;
use alloc::vec::Vec;
use area::{MapArea, Permission, MapType};
use pt::PageTable;

// 出于简单考虑，我第一步计划是将整个内存空间算作一个 map aera
const KERNEL_START_ADDR: usize = 0x80020000;
const KERNEL_END_ADDR: usize = 0x80200000;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
// 暂定将内核栈固定在 0x9000000 这个虚拟地址，大小为 8KiB，其实地址范围是 [0x90000000 - 8KiB, 0x90000000}
// 而且由于这一大段下面一直到内核空间都是无人使用的，相当于是一个保护页
const KERNEL_STACK_START: usize = 0x9000000;
// The memory manager for a process
pub struct MemoryManager {
    pt: PageTable,
    // 用 vec 的话，无法实现 dealloc 功能，
    // 分配出去的 maparea 如何回收是个很大的问题
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

        // 默认创建一个 8kib 的放置内核栈的 area，所以初始化 sp 可以设为 0x8FFFFFF0
        let mut kernel_stack_area = MapArea::new(
            (KERNEL_STACK_START- KERNEL_STACK_SIZE).into(), 
            (KERNEL_STACK_START).into(),
            MapType::Framed, 
            Permission::R | Permission::W
        );

        kernel_stack_area.map(&mut pt);

        Self { pt, areas: vec![kernel_area, kernel_stack_area] }
    }

    pub fn load_elf(&mut self) {
        // 根据 elf 文件生成 MapArea
    }
}
