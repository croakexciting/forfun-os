use alloc::string::String;

use crate::file::File;

pub mod qemu_blk;

pub trait BlockDevice: Send + Sync + File {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) -> Result<usize, String>;
    fn write_block(&self, block_id: usize, buf: &[u8]) -> Result<usize, String>;
    fn block_id(&self) -> usize;
}