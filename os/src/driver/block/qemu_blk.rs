use core::ptr::NonNull;

use alloc::string::{String, ToString};
use virtio_drivers::{
    device::blk::VirtIOBlk, 
    transport::{
        mmio::{MmioTransport, VirtIOHeader}, 
        DeviceType, Transport
    }, 
    BufferDirection, Hal, PhysAddr
};

use crate::{
    arch::memory::page::kernel_phys_to_virt, 
    mm::{dma::{dma_alloc, dma_dealloc}, pt::translate}
};

use super::BlockDevice;

pub struct QemuBlk {
    device: VirtIOBlk<HalImpl, MmioTransport>,
    block_size_log2: u8,
}

impl QemuBlk {
    pub fn new(addr: usize) -> Self {
        Self { device: init_blk(addr).unwrap(), block_size_log2: 9 }
    }
}

impl BlockDevice for QemuBlk {
    fn write_block(&mut self, block_id: usize, buf: &[u8]) -> Result<usize, String> {
        match self.device.write_blocks(block_id, buf) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(e.to_string())
        }
    }

    fn read_block(&mut self, block_id: usize, buf: &mut [u8]) -> Result<usize, String> {
        match self.device.read_blocks(block_id, buf) {
            Ok(_) => Ok(buf.len()),
            Err(e) => Err(e.to_string())
        }
    }

    fn block_size_log2(&self) -> u8 {
        self.block_size_log2
    }
}

pub fn init_blk(addr: usize) -> Option<VirtIOBlk<HalImpl, MmioTransport>> {
    let header = NonNull::new(addr as *mut VirtIOHeader).unwrap();
    match unsafe { MmioTransport::new(header) } {
        Err(e) => {
            println!("Error creating VirtIO MMIO transport: {}", e);
            None
        }
        Ok(transport) => {
            println!(
                "[kernel] Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
                transport.vendor_id(),
                transport.device_type(),
                transport.version(),
            );
            match transport.device_type() {
                DeviceType::Block => {
                    let blk = VirtIOBlk::<HalImpl, _>::new(transport).expect("failed to create blk driver");
                    Some(blk)
                }
                t => {
                    println!("Unrecognized virtio device: {:?}", t);
                    None
                }
            }
        }
    }
}
pub struct HalImpl;

unsafe impl Hal for HalImpl {
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let paddr: usize = dma_alloc(pages).unwrap();
        let vaddr: usize = kernel_phys_to_virt(paddr.into()).0;
        let ptr = if let Some(a) = NonNull::new(vaddr as *mut u8) {
            a
        } else {
            panic!("ptr allocator failed")
        };
        (paddr, ptr)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> 
    i32 {
        dma_dealloc(paddr, pages);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        let vaddr = kernel_phys_to_virt(paddr.into());
        NonNull::new(vaddr.0 as _).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let va = buffer.as_ptr() as *mut u8 as usize;
        if let Some(pa) = translate(va.into()) {
            return pa.0;
        } else {
            return 0;
        }
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
        // Nothing to do, as the host already has access to all memory and we didn't copy the buffer
        // anywhere else.
    }
}