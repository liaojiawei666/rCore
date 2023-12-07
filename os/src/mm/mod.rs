mod heap_allocator;
mod address;
mod frame_allocator;
mod page_table;
mod memory_set;
pub use memory_set::{MemorySet,MapPermission};
pub use address::{PhysPageNum,VirtAddr,PhysAddr,StepByOne}; 
pub use page_table::{translated_byte_buffer,translated_str,translated_refmut,UserBuffer,PageTable};
pub use heap_allocator::heap_test;
pub use memory_set::KERNEL_SPACE;
pub use memory_set::remap_test;
pub use frame_allocator::{frame_alloc,frame_dealloc,FrameTracker};
pub fn kernel_token() -> usize{
    KERNEL_SPACE.exclusive().token()
}
pub fn init(){
    heap_allocator::init_heap();
    frame_allocator::init_frame_allocator();
    KERNEL_SPACE.exclusive().activate();
}

