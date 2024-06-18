pub mod pt;
pub mod basic;
pub mod allocator;
pub mod area;
pub mod elf;

use alloc::vec;
use alloc::vec::Vec;
use area::{MapArea, Permission, MapType};
use basic::{PhysAddr, PhysPage, VirtAddr, VirtPage};
use pt::PageTable;

use crate::trap::context::TrapContext;

// 出于简单考虑，我第一步计划是将整个内存空间算作一个 map aera
const KERNEL_START_ADDR: usize = 0x80200000;
const KERNEL_END_ADDR: usize = 0x80400000;
const KERNEL_STACK_SIZE: usize = 4096 * 2;
// 暂定将内核栈固定在 0x9000000 这个虚拟地址，大小为 8KiB，其实地址范围是 [0x90000000 - 8KiB, 0x90000000}
// 而且由于这一大段下面一直到内核空间都是无人使用的，相当于是一个保护页
const KERNEL_STACK_START: usize = 0x90000000;

// 用户栈大小暂固定为 8KiB
const USER_STACK_SIZE: usize = 4096 * 2;

// The memory manager for a process
pub struct MemoryManager {
    pt: PageTable,
    _kernel_area: MapArea,
    kernel_stack_area: MapArea,
    _device_area: MapArea,
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

        // 将外设地址也 map 过去
        let mut device_area = MapArea::new(
            VirtAddr::from(0x1000_0000), 
            VirtAddr::from(0x1000_1000), 
            MapType::Identical,
            Permission::R | Permission::W | Permission::X
        );

        device_area.map(&mut pt);
        
        Self {
            pt,
            _kernel_area: kernel_area,
            kernel_stack_area, 
            _device_area: device_area,
            app_areas: vec![], 
        }
    }

    // return kernel stack pointer
    pub fn push_context(&mut self, ctx: TrapContext) -> usize {
        let sp_pa = self.pt.find_pte(self.kernel_stack_area.end_vpn.prev()).unwrap().ppn().next();

        let trap_ctx_ptr = (PhysAddr::from(sp_pa).0 - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_ctx_ptr = ctx;
        }

        let trap_ctx_ptr_va = VirtAddr::from(self.kernel_stack_area.end_vpn).0 - core::mem::size_of::<TrapContext>();
        trap_ctx_ptr_va as usize
    }

    pub fn runtime_pull_context(&mut self) -> TrapContext {
        let trap_ctx_ptr = (KERNEL_STACK_START - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe { 
            let ctx = (*trap_ctx_ptr).clone();
            ctx
        }
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
        let mut stack_area = MapArea::new(
            user_stack_bottom, 
            user_stack_top, 
            MapType::Framed, 
            Permission::R | Permission::W | Permission::U | Permission::X
        );
        stack_area.map(&mut self.pt);
        self.app_areas.push(
            stack_area
        );

        Ok((user_stack_top.0 - 0x100, elf.header.pt2.entry_point() as usize))
    }

    pub fn fork(&mut self, parent: &mut Self) {
        // 将父进程的所有 app area 复制一份，并且为 physframe 计数 +1
        self.app_areas.reserve(parent.app_areas.len());
        for area in parent.app_areas.iter() {
            let new_area = area.fork(&mut parent.pt, &mut self.pt);
            self.app_areas.push(new_area);
        }

        let ctx = parent.runtime_pull_context();
        let kernel_stack_pa = self.pt.translate_ceil(
            // 所有的 memory area 都是一个左闭右开的范围，所以 end_vpn 是不被包括在内的
            // 也就是说 end_vpn 的起始位置就是地址范围的右端，同样是不被包括在内
            self.kernel_stack_area.end_vpn.into()
        ).unwrap();
        // 由于 kernel stack start 是不被包括在内的，所以需要 -1 后的地址才是实际需要 map 的虚拟地址
        parent.pt.kmap(kernel_stack_pa.reduce(1));
        let trap_ctx_ptr = (kernel_stack_pa.0 - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_ctx_ptr = ctx;
            (*trap_ctx_ptr).x[10] = 0;
        }
        parent.pt.kunmap(kernel_stack_pa.reduce(1));
    }

    pub fn root_ppn(&self) -> PhysPage {
        self.pt.root_ppn()
    }

    pub fn remap(&mut self, vpn: VirtPage) -> Result<(), &'static str> {
        for m in self.app_areas.iter_mut() {
            if m.start_vpn.0 <= vpn.0 && vpn.0 < m.end_vpn.0 {
                return m.remap(&mut self.pt, vpn)
            }
        }

        Err("vpn is not in this memory set")
    }
}
