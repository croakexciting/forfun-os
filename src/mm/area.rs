// 实现创建一个 app 时所需所有的内存空间的创建和映射
// 处于简单考虑，暂且将内核空间所有段，包括堆和栈空间，直连到物理空间，后续需要优化 allocator 实现堆的动态扩容
use alloc::sync::Arc;
use bitflags::bitflags;
use alloc::collections::BTreeMap;

use super::{
    allocator::{frame_alloc, PhysFrame}, 
    basic::{
        PTEFlags, PageTableEntry, PhysPage, 
        VirtAddr, VirtPage, 
        PAGE_SIZE,
    }, 
    pt::PageTable};

#[derive(Clone)]
pub struct MapArea {
    pub start_vpn: VirtPage,
    // end_vpn 是不包含在内的，也就是一个左闭右开的范围
    pub end_vpn: VirtPage,
    
    map_type: MapType,
    permission: Permission,
    // 放在这里只是为了在 drop 的时候自动执行 dealloc 回收这些物理页帧到 alloctor
    // virtual page => physframe
    frames: BTreeMap<usize, Arc<PhysFrame>>,
}

// 简单设计，一个 map area 中的内存页帧是一起创建，一起消失的。同时起始位置必须 4K对齐
// 比如 app 中一次 malloc 就会产生一个 map area，后续考虑类似 brk 的功能
impl MapArea {
    pub fn new(
        start_va: VirtAddr,
        end_va: VirtAddr, 
        map_type: MapType, 
        permission: Permission
    ) -> Self {
        Self {
            start_vpn: start_va.into(),
            end_vpn: VirtAddr::from(end_va.0 - 1 + PAGE_SIZE).into(),
            frames: BTreeMap::new(),
            map_type,
            permission,
        }
    }

    pub fn map_one(&mut self, pt: &mut PageTable, vpn: VirtPage) -> Option<PageTableEntry> {
        let ppn: PhysPage;
        match self.map_type {
            MapType::Identical => {
                // 恒等映射，用于内核内存（除了对应 app 的 kernel stack）
                // 这些内存是与 app 完全无关的，只是放到每个 app 的页表中，避免其进入内核态后，无法正确运行
                ppn = PhysPage(vpn.0);
            }
            MapType::Framed => {
                let frame = frame_alloc()?;
                ppn = frame.ppn;
                self.frames.insert(vpn.0, Arc::new(frame));
            }
        }
        let pte_flag = PTEFlags::from_bits(self.permission.bits())?;
        pt.map(vpn, ppn, pte_flag)
    }

    pub fn unmap_one(&mut self, pt: &mut PageTable, vpn: VirtPage) -> i32 {
        self.frames.remove(&vpn.0);
        pt.unmap(vpn)
    }

    pub fn map(&mut self, pt: &mut PageTable) -> i32 {
        for v in self.start_vpn.0..self.end_vpn.0 {
            self.map_one(pt, v.into());
        }

        return 0;
    }

    pub fn map_with_data(&mut self, pt: &mut PageTable, data: &[u8]) -> Result<(), &'static str>{
        if data.len() > (self.end_vpn.0 - self.start_vpn.0) * PAGE_SIZE {
            return Err("data length overflow");
        }

        let mut offset: usize = 0;
        for v in self.start_vpn.0..self.end_vpn.0 {
            // map
            let pte = self.map_one(pt, v.into());

            // copy data page by page
            if let Some(p) = pte {
                let src = &data[offset..data.len().min(offset + PAGE_SIZE)];
                let dst = &mut p.ppn().bytes_array()[..src.len()];
                dst.copy_from_slice(src);
                offset += PAGE_SIZE;
            } else {
                return Err("pte map failed");
            }
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn unmap(&mut self, pt: &mut PageTable) -> i32 {
        for v in self.start_vpn.0..self.end_vpn.0 {
            self.unmap_one(pt, v.into());
        }

        return 0;
    }

    pub fn fork(&self, pt: &mut PageTable, child_pt: &mut PageTable) -> Self {
        let mut child_frames: BTreeMap<usize, Arc<PhysFrame>> = BTreeMap::new();

        for (k, v) in self.frames.iter() {
            let pte = pt.find_valid_pte((*k).into()).unwrap();
            let mut flags = pte.flags().unwrap();
            flags.remove(PTEFlags::W);

            child_pt.map((*k).into(), v.ppn, flags).unwrap();
            pt.remap((*k).into(), v.ppn, flags).unwrap();
            child_frames.insert(*k, v.clone());
        }
        
        Self {
            start_vpn: self.start_vpn,
            end_vpn: self.end_vpn,
            map_type: self.map_type,
            permission: self.permission,
            frames: child_frames,
        } 
    }

    pub fn remap(&mut self, pt: &mut PageTable, vpn: VirtPage) -> Result<(), &'static str> {
        if (self.unmap_one(pt, vpn)) < 0 {
            return Err("unmap failed");
        }

        if let Some(_) = self.map_one(pt, vpn) {
            Ok(())
        } else {
            return Err("remap failed");
        }
    }
}

#[derive(Copy, Clone, Debug)]
/// map type for memory set: identical or framed
pub enum MapType {
    Identical,
    Framed,
}

bitflags! {
    /// map permission corresponding to that in pte: `R W X U`
    #[derive(Copy, Clone)]
    pub struct Permission: u8 {
        const R = 1 << 1;
        const W = 1 << 2;
        const X = 1 << 3;
        const U = 1 << 4;
    }
}