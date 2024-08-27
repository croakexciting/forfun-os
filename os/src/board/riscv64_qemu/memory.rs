// kernel code space
pub const KERNEL_START_ADDR: usize = 0x80200000;
pub const KERNEL_END_ADDR: usize = 0x80400000;

// kernel stack
pub const KERNEL_STACK_START: usize = 0x80000000;
pub const KERNEL_STACK_SIZE: usize = 4096 * 16;

// user stack
pub const USER_STACK_START: usize = 0x7ff8_0000;
pub const USER_STACK_SIZE: usize = 4096 * 2;

// memory allocator area
pub const KERNEL_ALLOCATOR_START: usize = 0x80380000;
pub const ALLOCATOR_START: usize = 0x80400000;
pub const ALLOCATOR_END: usize = 0x80800000;

// dma area
pub const DMA_START_ADDR: usize = 0x8600_0000;
pub const DMA_END_ADDR: usize = 0x8700_0000;

// peripheral area
pub const PERIPHERAL_START_ADDR: usize = 0x9000_0000;
pub const PERIPHERAL_END_ADDR: usize = 0xA000_0000;