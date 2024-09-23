pub mod pt;
pub mod allocator;
pub mod area;
pub mod elf;
pub mod buddy;
pub mod dma;
pub mod peripheral;

use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;
use area::{MapArea, Permission, MapType};
use peripheral::peripheral_alloc;
use crate::arch::memory::page::{
    PageTableEntry, PhysAddr, PhysPage, VirtAddr, VirtPage, PAGE_SIZE
};
use buddy::BuddyAllocator;
use pt::PageTable;
use spin::rwlock::RwLock;

use crate::arch::context::TrapContext;
use crate::board::peri::memory::*;

// The memory manager for a process
pub struct MemoryManager {
    pub pt: PageTable,
    kernel_stack_area: MapArea,
    app_areas: Vec<Arc<RwLock<MapArea>>>,
    // 堆可用区域，左闭右开的集合
    buddy_alloctor: Option<BuddyAllocator>,
    kernel_pte: PageTableEntry,

    _kernel_area: Vec<MapArea>,
}

impl MemoryManager {
    pub fn new(if_kernel: bool) -> Self {
        let mut pt = PageTable::new();
        let app_areas: Vec<Arc<RwLock<MapArea>>> = Vec::with_capacity(8);
        let _kernel_area = Vec::new();

        let mut kernel_stack_area = MapArea::new(
            (KERNEL_STACK_START- KERNEL_STACK_SIZE).into(), 
            (KERNEL_STACK_START).into(),
            MapType::Framed, 
            Permission::R | Permission::W
        );

        if !if_kernel {
            kernel_stack_area.map(&mut pt);
        }
        
        Self {
            pt,
            kernel_stack_area,
            app_areas,
            buddy_alloctor: None,
            kernel_pte: PageTableEntry(0),
            _kernel_area,
        }
    }

    fn alloc(&mut self, pn: usize) -> Option<(VirtPage, VirtPage)> {
        if let Some(allocator) = &mut self.buddy_alloctor {
            let start = allocator.alloc(pn)?;
            Some((start, start.add(pn)))
        } else {
            None
        }
    }

    fn dealloc(&mut self, area: &Arc<RwLock<MapArea>>) {
        let vpn = area.read().start_vpn;
        let pn = area.read().end_vpn.0 - area.read().start_vpn.0;
        if let Some(allocator) = &mut self.buddy_alloctor {
            allocator.dealloc(vpn, pn)
        }
    }

    // return kernel stack pointer
    pub fn push_context(&mut self, ctx: TrapContext, current_pt: &mut PageTable) -> usize {
        let sp_pa = self.pt.find_pte(self.kernel_stack_area.end_vpn.prev()).unwrap().ppn().next();

        let trap_ctx_ptr = (PhysAddr::from(sp_pa).0 - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        current_pt.kmap(PhysAddr(trap_ctx_ptr as usize));
        unsafe {
            *trap_ctx_ptr = ctx;
        }
        current_pt.kunmap(PhysAddr(trap_ctx_ptr as usize));

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
    
    pub fn load_elf(&mut self, current_pt: &mut PageTable, data: &[u8], runtime: bool) -> Result<(usize, usize), &'static str>{
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
                        &mut self.pt, current_pt,
                        &elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize])?;
                }
                offset = area.end_vpn;
                self.app_areas.push(Arc::new(RwLock::new(area)));
            }
        }

        // 添加一个保护页
        let start_vpn = offset.next();
        // 预留 512M，也就是 32*1000 个页
        self.buddy_alloctor = Some(BuddyAllocator::new(10, start_vpn, 32*1000));

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
            Arc::new(RwLock::new(stack_area))
        );

        Ok((user_stack_top.0 - 0x100, elf.header.pt2.entry_point() as usize))
    }

    pub fn fork(&mut self, parent: &mut Self) {
        // append kernel stack
        self.add_kernel_pt(parent);
        
        // 将父进程的所有 app area 复制一份，通过智能指针来实现自动 drop
        self.app_areas.reserve(parent.app_areas.len());
        for area in parent.app_areas.iter_mut() {
            let new_area = area.write().fork(&mut parent.pt, &mut self.pt);
            self.app_areas.push(Arc::new(RwLock::new(new_area)));
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
        self.buddy_alloctor = parent.buddy_alloctor.clone();
    }

    pub fn root_ppn(&self) -> PhysPage {
        self.pt.root_ppn()
    }

    pub fn cow(&mut self, vpn: VirtPage) -> Result<(), &'static str> {
        for m in self.app_areas.iter_mut() {
            if m.read().start_vpn.0 <= vpn.0 && vpn.0 < m.read().end_vpn.0 {
                return m.write().cow(&mut self.pt, vpn)
            }
        }

        println!("[kernel] vpn {} is not in memoryset", vpn.0);
        Err("vpn is not in this memory set")
    }

    pub fn unmap_app(&mut self) {
        for area in self.app_areas.iter_mut() {
            area.write().unmap(&mut self.pt);
        }

        self.app_areas.clear();
    }

    pub fn mmap(&mut self, size: usize, permission: usize) -> Option<Weak<RwLock<MapArea>>> {
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
        let area_ptr = Arc::new(RwLock::new(new_area));
        let weak_ptr = Arc::downgrade(&area_ptr);
        self.app_areas.push(area_ptr);

        Some(weak_ptr)
    }

    pub fn mmap_with_addr(&mut self, pa: PhysAddr, size: usize, permission: usize, user: bool) -> isize {
        assert_eq!(size % PAGE_SIZE, 0);

        let mut ppns: Vec<PhysPage> = Vec::new();

        let mut p = Permission::from_bits_truncate((permission as u8) << 1);

        if user == true {
            p.insert(Permission::U);
        }

        for i in 0..(size/PAGE_SIZE) {
            let mut ppn: PhysPage = pa.into();
            ppn = ppn.add(i);
            ppns.push(ppn)
        }

        self.map_defined(&ppns, p, user)
    }

    pub fn umap_dyn_area(&mut self, start_vpn: VirtPage) -> isize {
        if let Some(index) = self.app_areas.iter().position(|a| a.read().start_vpn == start_vpn) {
            let area  = self.app_areas.remove(index);
            area.write().unmap(&mut self.pt);
            self.dealloc(&area);
            return 0;
        }

        // not find
        println!("[kernel] can't find map area start with {:#x}", start_vpn.0);
        -1
    }

    pub fn map_defined(&mut self, ppns: &Vec<PhysPage>, permission: Permission, user: bool) -> isize {
        if let Some(vpns) = self.alloc(ppns.len()) {
            let mut new_area = MapArea::new(
                vpns.0.into(), 
                vpns.1.into(), 
                MapType::Defined,
                permission.clone()
            );
            new_area.map_defined(&mut self.pt, ppns);
            if user == true {
                let area_ptr = Arc::new(RwLock::new(new_area));
                self.app_areas.push(area_ptr);    
            } else {
                self._kernel_area.push(new_area);
            }
            VirtAddr::from(vpns.0).0 as isize
        } else {
            -1
        }
    }

    pub fn map_peripheral(&mut self, pa: PhysAddr, size: usize) -> isize {
        let p = Permission::R | Permission::W | Permission::X;

        let mut pn = size / PAGE_SIZE;
        if (size % PAGE_SIZE) != 0 {
            pn += 1;
        }

        let mut ppns: Vec<PhysPage> = Vec::with_capacity(pn);
        let mut start_ppn: PhysPage = pa.into();
        for _ in 0..pn {
            ppns.push(start_ppn);
            start_ppn = start_ppn.next();
        }

        if let Some(vpa_start) = peripheral_alloc(pn) {
            let mut new_area = MapArea::new(
                vpa_start.into(), 
                (vpa_start + size).into(), 
                MapType::Defined, 
                p
            );
            new_area.map_defined(&mut self.pt, &ppns);
            self._kernel_area.push(new_area);
            vpa_start as isize
        } else {
            -1
        }
    }

    /*
        如果每个进程的页表都管理一套 kernel area，相当浪费内存
        计划是在 0 级（最高级）页表上留一个页表项指向一个 1 级页表
        这个页表存储 kernel pte，每个进程 0 级页表均指向它，这样实现了 kernel pte 共享
        一个一级页表可以指向 1G 内存，内核空间也够用，kernel 和外设地址都放在这里
    */
    pub fn init_kernel_area(&mut self) -> Option<PageTableEntry> {
        let mut kcode = MapArea::new(
            KERNEL_START_ADDR.into(),
            KERNEL_END_ADDR.into(),
            MapType::Identical,
            Permission::R | Permission::W | Permission::X
        );

        kcode.map(&mut self.pt);
        self._kernel_area.push(kcode);

        let mut dma_area = MapArea::new(
            VirtAddr::from(DMA_START_ADDR), 
            VirtAddr::from(DMA_END_ADDR), 
            MapType::Identical,
            Permission::R | Permission::W
        );
        dma_area.map(&mut self.pt);
        self._kernel_area.push(dma_area);

        let pte = self.pt.find_pte_with_level(VirtAddr(KERNEL_START_ADDR).into(), 0, true)?;
        self.kernel_pte = pte.clone();
        Some(self.kernel_pte)
    }

    pub fn add_kernel_pt(&mut self, parent: &mut Self) -> Option<PageTableEntry> {
        self.kernel_pte = parent.kernel_pte;
        self.pt.set_pte_with_level(parent.kernel_pte, VirtAddr(KERNEL_START_ADDR).into(), 0)
    }
}
