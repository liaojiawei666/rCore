mod heap_allocator;
mod address;
mod frame_allocator;
mod page_table;
mod memory_set;
pub use memory_set::{MemorySet,MapPermission};
pub use address::{PhysPageNum,VirtAddr}; 
pub use page_table::translated_byte_buffer;
pub use heap_allocator::heap_test;
pub use memory_set::KERNEL_SPACE;
pub fn init(){
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive().activate();
}

