pub mod pt;
pub mod basic;
pub mod allocator;
pub mod area;
pub mod elf;

use alloc::vec;
use alloc::vec::Vec;
use area::{MapArea, Permission, MapType};
use basic::{PhysPage, VirtAddr, VirtPage};
use pt::PageTable;

use crate::trap::context::TrapContext;

// 出于简单考虑，我第一步计划是将整个内存空间算作一个 map aera
const KERNEL_START_ADDR: usize = 0x80020000;
const KERNEL_END_ADDR: usize = 0x80200000;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
// 暂定将内核栈固定在 0x9000000 这个虚拟地址，大小为 8KiB，其实地址范围是 [0x90000000 - 8KiB, 0x90000000}
// 而且由于这一大段下面一直到内核空间都是无人使用的，相当于是一个保护页
const KERNEL_STACK_START: usize = 0x9000000;

// 用户栈大小暂固定为 8KiB
const USER_STACK_SIZE: usize = 4096 * 2;

// The memory manager for a process
pub struct MemoryManager {
    pt: PageTable,
    _kernel_area: MapArea,
    kernel_stack_area: MapArea,
    // 用 vec 的话，无法实现 dealloc 功能，
    // 分配出去的 maparea 如何回收是个很大的问题
    app_areas: Vec<MapArea>,
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

        Self {
            pt,
            _kernel_area: kernel_area,
            kernel_stack_area, 
            app_areas: vec![], 
        }
    }

    // return kernel stack pointer
    pub fn push_context(&self, ctx: TrapContext) -> usize {
        let sp: VirtAddr = self.kernel_stack_area.end_vpn.into();
        let trap_ctx_ptr = (sp.0 - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_ctx_ptr = ctx;
        }
        trap_ctx_ptr as usize

    }

    pub fn load_elf(&mut self, data: &[u8]) -> Result<(usize, usize), &'static str>{
        // 根据 elf 文件生成 MapArea
        let elf = elf::parse(data)?;
        let ph_count = elf.header.pt2.ph_count();
        let mut offset = VirtPage(0);
        for i in 0..ph_count {
            let ph = elf.program_header(i)?;
            let ph_type = ph.get_type()?;
            if ph_type == xmas_elf::program::Type::Load {
                let start_va: VirtAddr = (ph.virtual_addr() as usize).into();
                let end_va: VirtAddr = ((ph.virtual_addr() + ph.mem_size()) as usize).into();
                let mut permission = Permission::U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    permission |= Permission::R;
                }
                if ph_flags.is_write() {
                    permission |= Permission::W;
                }
                if ph_flags.is_execute() {
                    permission |= Permission::X;
                }
                let mut area = MapArea::new(start_va, end_va, MapType::Framed, permission);
                // copy data from elf into map area
                area.map_with_data(
                    &mut self.pt, 
                    &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])?;
                offset = area.end_vpn;
                self.app_areas.push(area);
            }
        }

        // 应用程序栈放在一个保护页之后，这样栈溢出的时候就会报页错误
        offset = offset.next();

        let user_stack_bottom: VirtAddr = offset.into();
        let user_stack_top = user_stack_bottom.add(USER_STACK_SIZE);
        self.app_areas.push(
            MapArea::new(
                user_stack_bottom, 
                user_stack_top, 
                MapType::Framed,
                Permission::R | Permission::W | Permission::U
            )
        );

        Ok((user_stack_top.0, elf.header.pt2.entry_point() as usize))
    }

    pub fn root_ppn(&self) -> PhysPage {
        self.pt.root_ppn()
    }
}