// kernel code space - Identical
pub const KERNEL_START_ADDR: usize = 0x4000_0000;
pub const KERNEL_END_ADDR: usize = 0x4040_0000;

// kernel stack - Framed
pub const KERNEL_STACK_START: usize = 0x1_0000_0000;
pub const KERNEL_STACK_SIZE: usize = 4096 * 16;

// user stack - Framed
pub const USER_STACK_START: usize = 0x4000_0000;
pub const USER_STACK_SIZE: usize = 4096 * 2;

// memory allocator area - Physical memory page
pub const KERNEL_ALLOCATOR_START: usize = 0x4038_0000;
pub const ALLOCATOR_START: usize = 0x4040_0000;
pub const ALLOCATOR_END: usize = 0x4080_0000;

// dma area - Identical
pub const DMA_START_ADDR: usize = 0x4600_0000;
pub const DMA_END_ADDR: usize = 0x4700_0000;

// peripheral area - Defined
pub const PERIPHERAL_START_ADDR: usize = 0x5000_0000;
pub const PERIPHERAL_END_ADDR: usize = 0x6000_0000;

// initproc load addr
pub const INITPROC_LOAD_ADDR: usize = 0x4100_0000;