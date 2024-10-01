// kernel stack - Framed
pub const KERNEL_STACK_START: usize = 0x1_0000_0000;
pub const KERNEL_STACK_SIZE: usize = 4096 * 16;

// user stack - Framed
pub const USER_STACK_START: usize = 0xFFF8_0000;
pub const USER_STACK_SIZE: usize = 4096 * 2;

// memory allocator area - Physical memory page
pub const KERNEL_ALLOCATOR_START: usize = 0x80400000;
pub const ALLOCATOR_START: usize = 0x81000000;
pub const ALLOCATOR_END: usize = 0x87000000;

// dma area - Identical
pub const DMA_START_ADDR: usize = 0x8700_0000;
pub const DMA_END_ADDR: usize = 0x8800_0000;
