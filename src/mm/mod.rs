pub mod pt;
pub mod basic;
pub mod allocator;
pub mod area;
pub mod elf;

use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use area::{MapArea, Permission, MapType};
use basic::{PhysAddr, PhysPage, VirtAddr, VirtPage, PAGE_SIZE};
use pt::PageTable;
use spin::mutex::Mutex;

use crate::trap::context::TrapContext;

// 出于简单考虑，我第一步计划是将整个内存空间算作一个 map aera
const KERNEL_START_ADDR: usize = 0x80200000;
const KERNEL_END_ADDR: usize = 0x80400000;
const KERNEL_STACK_SIZE: usize = 4096 * 4;
// 暂定将内核栈固定在 0x9000000 这个虚拟地址，大小为 16KiB，其实地址范围是 [0x90000000 - 16KiB, 0x90000000}
// 而且由于这一大段下面一直到内核空间都是无人使用的，相当于是一个保护页
const KERNEL_STACK_START: usize = 0x90000000;
const USER_STACK_START: usize = 0x80000000;

// 用户栈大小暂固定为 8KiB
const USER_STACK_SIZE: usize = 4096 * 2;

// The memory manager for a process
pub struct MemoryManager {
    pt: PageTable,
    _kernel_area: MapArea,
    kernel_stack_area: MapArea,
    _device_area: MapArea,
    app_areas: Vec<Arc<Mutex<MapArea>>>,
    // 堆可用区域，左闭右开的集合
    available: (VirtPage, VirtPage),
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

        let mut elf_area = MapArea::new(
            VirtAddr::from(0x8200_0000),
            VirtAddr::from(0x8205_0000),
            MapType::Identical,
            Permission::R | Permission::W | Permission::X
        );
        elf_area.map(&mut pt);

        let app_areas: Vec<Arc<Mutex<MapArea>>> = Vec::with_capacity(8);
        
        Self {
            pt,
            _kernel_area: kernel_area,
            kernel_stack_area, 
            _device_area: device_area,
            app_areas,
            available: (0.into(), 0.into()),
        }
    }

    fn alloc(&mut self, pn: usize) -> Option<(VirtPage, VirtPage)> {
        // alloc 需要优化，支持 dealloc 功能
        if (self.available.0.0 + pn) > self.available.1.0 {
            println!("[kernel] virtual memory space run out");
            return None;
        }

        self.available.0.add(pn);

        return Some((self.available.0, self.available.0.add(pn)));
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

    pub fn runtime_push_context(&mut self, ctx: TrapContext) -> usize {
        let trap_ctx_ptr = (KERNEL_STACK_START - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *trap_ctx_ptr = ctx;
        }

        trap_ctx_ptr as usize
    }
    
    pub fn load_elf(&mut self, data: &[u8], runtime: bool) -> Result<(usize, usize), &'static str>{
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
                if runtime == true {
                    area.runtime_map_with_data(
                        &mut self.pt, 
                        &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])?;
                } else {
                    area.map_with_data(
                        &mut self.pt, 
                        &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])?;
                }
                offset = area.end_vpn;
                self.app_areas.push(Arc::new(Mutex::new(area)));
            }
        }

        // 添加一个保护页
        self.available.0 = offset.next();

        let user_stack_top: VirtAddr = USER_STACK_START.into();
        let user_stack_bottom: VirtAddr = user_stack_top.reduce(USER_STACK_SIZE);
        let mut stack_area = MapArea::new(
            user_stack_bottom, 
            user_stack_top, 
            MapType::Framed, 
            Permission::R | Permission::W | Permission::U | Permission::X
        );
        stack_area.map(&mut self.pt);
        self.app_areas.push(
            Arc::new(Mutex::new(stack_area))
        );

        // 栈下面添加一个保护页，栈溢出时报错
        let stack_bottom_vpn: VirtPage = user_stack_bottom.into();
        self.available.1 = stack_bottom_vpn.prev();

        Ok((user_stack_top.0 - 0x100, elf.header.pt2.entry_point() as usize))
    }

    pub fn fork(&mut self, parent: &mut Self) {
        // 将父进程的所有 app area 复制一份，并且为 physframe 计数 +1
        self.app_areas.reserve(parent.app_areas.len());
        for area in parent.app_areas.iter_mut() {
            let new_area = area.lock().fork(&mut parent.pt, &mut self.pt);
            self.app_areas.push(Arc::new(Mutex::new(new_area)));
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
        self.available = parent.available;
    }

    pub fn root_ppn(&self) -> PhysPage {
        self.pt.root_ppn()
    }

    pub fn cow(&mut self, vpn: VirtPage) -> Result<(), &'static str> {
        for m in self.app_areas.iter_mut() {
            if m.lock().start_vpn.0 <= vpn.0 && vpn.0 < m.lock().end_vpn.0 {
                return m.lock().cow(&mut self.pt, vpn)
            }
        }

        println!("[kernel] vpn {} is not in memoryset", vpn.0);
        Err("vpn is not in this memory set")
    }

    pub fn unmap_app(&mut self) {
        for area in self.app_areas.iter_mut() {
            area.lock().unmap(&mut self.pt);
        }

        self.app_areas.clear();
    }

    pub fn mmap(&mut self, size: usize, permission: usize) -> Option<Weak<Mutex<MapArea>>> {
        assert_eq!(size % PAGE_SIZE, 0);

        let mut p = Permission::from_bits_truncate((permission as u8) << 1);
        p.insert(Permission::U);

        let (start_vpn, end_vpn) = self.alloc(size / PAGE_SIZE)?;

        let mut new_area = MapArea::new(
            start_vpn.into(),
            end_vpn.into(),
            MapType::Framed,
            p
        );
        new_area.map(&mut self.pt);
        let area_ptr = Arc::new(Mutex::new(new_area));
        let weak_ptr = Arc::downgrade(&area_ptr);
        self.app_areas.push(area_ptr);

        Some(weak_ptr)
    }

    pub fn map_defined(&mut self, ppns: &Vec<PhysPage>, permission: Permission) -> isize {
        if let Some(vpns) = self.alloc(ppns.len()) {
            let mut new_area = MapArea::new(
                vpns.0.into(), 
                vpns.1.into(), 
                MapType::Defined,
                permission.clone()
            );
            new_area.map_defined(&mut self.pt, ppns);
            VirtAddr::from(vpns.0).0 as isize
        } else {
            -1
        }
    }
}
