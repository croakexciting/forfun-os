use alloc::string::String;

pub mod qemu_blk;

pub trait BlockDevice: Send + Sync {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> Result<(), String>;
    fn write_block(&self, block_id: usize, buf: &[u8]) -> Result<(), String>;
}