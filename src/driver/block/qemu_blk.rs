use core::ptr::NonNull;

use alloc::{string::{String, ToString}, vec::Vec};
use virtio_drivers::{
    device::blk::VirtIOBlk, 
    transport::{
        mmio::{MmioTransport, VirtIOHeader}, 
        DeviceType, Transport
    }, 
    BufferDirection, Hal, PhysAddr, PAGE_SIZE
    
};

use lazy_static::*;

const BLK_HEADER_ADDR: usize = 0x10008000;

use crate::{mm::{allocator::{frame_alloc, frame_dealloc, kernel_frame_alloc, kernel_frame_dealloc, PhysFrame}, basic::{self, PhysPage}, pt}, utils::type_extern::RefCellWrap};

use super::BlockDevice;

lazy_static! {
    static ref QUEUE_FRAMES: RefCellWrap<Vec<PhysFrame>> = unsafe { RefCellWrap::new(Vec::new()) };
}

lazy_static! {
    static ref BLOCK_DEVICE: QemuBlk = QemuBlk::new();
}
pub struct QemuBlk {
    inner: RefCellWrap<VirtIOBlk<HalImpl, MmioTransport>>
}

pub fn write_block(block_id: usize, buf: &[u8]) -> Result<(), String> {
    BLOCK_DEVICE.write_block(block_id, buf)
}

pub fn read_block(block_id: usize, buf: &mut [u8]) -> Result<(), String> {
    BLOCK_DEVICE.read_block(block_id, buf)
}

impl QemuBlk {
    pub fn new() -> Self {
        unsafe { Self { inner: RefCellWrap::new(init_blk().unwrap()) } }
    }
}

impl BlockDevice for QemuBlk {
    fn write_block(&self, block_id: usize, buf: &[u8]) -> Result<(), String> {
        match self.inner.exclusive_access().write_blocks(block_id, buf) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }

    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> Result<(), String> {
        match self.inner.exclusive_access().read_blocks(block_id, buf) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string())
        }
    }
}

pub fn init_blk() -> Option<VirtIOBlk<HalImpl, MmioTransport>> {
    let header = NonNull::new(BLK_HEADER_ADDR as *mut VirtIOHeader).unwrap();
    match unsafe { MmioTransport::new(header) } {
        Err(e) => {
            println!("Error creating VirtIO MMIO transport: {}", e);
            None
        }
        Ok(transport) => {
            println!(
                "Detected virtio MMIO device with vendor id {:#X}, device type {:?}, version {:?}",
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
        let mut ppn_base = PhysPage(0);
        for i in 0..pages {
            let frame = kernel_frame_alloc().unwrap();
            if i == 0 {
                ppn_base = frame.ppn;
            }
            assert_eq!(frame.ppn.0, ppn_base.0 + i);
            QUEUE_FRAMES.exclusive_access().push(frame);
        }
        let pa: basic::PhysAddr = ppn_base.into();
        let ptr = if let Some(a) = NonNull::new(pa.0 as *mut u8) {
            a
        } else {
            panic!("ptr allocator failed")
        };
        (pa.0, ptr)
    }

    unsafe fn dma_dealloc(paddr: PhysAddr, _vaddr: NonNull<u8>, pages: usize) -> 
    i32 {
        let pa = basic::PhysAddr::from(paddr);
        let mut ppn_base: PhysPage = pa.into();
        for _ in 0..pages {
            kernel_frame_dealloc(ppn_base);
            ppn_base = ppn_base.next();
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(paddr as _).unwrap()
    }

    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        buffer.as_ptr() as *mut u8 as PhysAddr
    }

    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {
        // Nothing to do, as the host already has access to all memory and we didn't copy the buffer
        // anywhere else.
    }
}