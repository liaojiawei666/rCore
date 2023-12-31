pub const APP_BASE_ADDRESS: usize = 0x80400000;
pub const APP_SIZE_LIMIT: usize = 0x20000; // 128KB

pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x30_0000; //3MB
pub const MAX_APP_NUM: usize = 16;
pub const CLOCK_FREQ: usize = 12500000;
pub const MEMORY_END: usize = 0x88000000;
pub const PAGE_SIZE: usize = 4096; //4KB
pub const PAGE_SIZE_BITS: usize = 0xc; //12
pub const TRAMPOLINE: usize = usize::MAX - PAGE_SIZE + 1;
pub const TRAP_CONTEXT: usize = TRAMPOLINE - PAGE_SIZE;

pub const MIMO: &[(usize, usize)] = &[(0x1000_1000, 0x1000)];
