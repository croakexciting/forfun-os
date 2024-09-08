// kernel code space - Identical
pub const KERNEL_START_ADDR: usize = 0x80200000;
pub const KERNEL_END_ADDR: usize = 0x80400000;

// kernel stack - Framed
pub const KERNEL_STACK_START: usize = 0x100000000;
pub const KERNEL_STACK_SIZE: usize = 4096 * 16;

// user stack - Framed
pub const USER_STACK_START: usize = 0x8000_0000;
pub const USER_STACK_SIZE: usize = 4096 * 2;

// memory allocator area - Physical memory page
pub const KERNEL_ALLOCATOR_START: usize = 0x80380000;
pub const ALLOCATOR_START: usize = 0x80400000;
pub const ALLOCATOR_END: usize = 0x80800000;

// dma area - Identical
pub const DMA_START_ADDR: usize = 0x8600_0000;
pub const DMA_END_ADDR: usize = 0x8700_0000;

// peripheral area - Defined
pub const PERIPHERAL_START_ADDR: usize = 0x9000_0000;
pub const PERIPHERAL_END_ADDR: usize = 0xA000_0000;

// initproc load addr
pub const INITPROC_LOAD_ADDR: usize = 0x8100_0000;