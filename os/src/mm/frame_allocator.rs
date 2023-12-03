use super::address::*;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::{sync::UPSafeCell, config::MEMORY_END};
type FrameAllocatorImpl=StackFrameAllocator;
lazy_static!{
    pub static ref FRAME_ALLOCATOR:UPSafeCell<FrameAllocatorImpl>=unsafe{
        UPSafeCell::new(FrameAllocatorImpl::new())
    };
}
trait FrameAllocator{
    fn new()->Self;
    fn alloc(&mut self)->Option<PhysPageNum>;
    fn dealloc(&mut self,ppn:PhysPageNum);
}

pub struct StackFrameAllocator{
    current:usize,
    end:usize,
    recycled:Vec<usize>,
}
impl FrameAllocator for StackFrameAllocator{
    fn new()->Self{
        Self{
            current:0,
            end:0,
            recycled:Vec::new(),
        }
    }
    fn alloc(&mut self)->Option<PhysPageNum>{
        if let Some(ppn)=self.recycled.pop(){
            Some(PhysPageNum(ppn))
        }else if self.current<self.end{
            let ppn=PhysPageNum(self.current);
            self.current+=1;
            Some(ppn)
        }else{
            None
        }
    }
    fn dealloc(&mut self,ppn:PhysPageNum) {
        let ppn=ppn.0;
        if ppn>=self.current||self.recycled.contains(&ppn){
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        self.recycled.push(ppn);
    }
}
impl StackFrameAllocator{
    fn init(&mut self,l:PhysPageNum,r:PhysPageNum){
        self.current=l.0;
        self.end=r.0;
    }
}

pub fn init_frame_allocator(){
    extern "C"{fn ekernel();}
    println!("ekernel: {:#x}",ekernel as usize);
    FRAME_ALLOCATOR.exclusive()
    .init(PhysAddr::from(ekernel as usize).ceil(),PhysAddr::from(MEMORY_END).floor());
}

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}
impl FrameTracker{
    pub fn new(ppn:PhysPageNum)->Self{
        let bytes_array=ppn.get_bytes_array();

        for i in bytes_array{
            *i=0;
        }
        Self{ppn}
    }
}
impl Drop for FrameTracker{
    fn drop(&mut self){
        frame_dealloc(self.ppn);
    }
}
pub fn frame_alloc()->Option<FrameTracker>{
    FRAME_ALLOCATOR.exclusive().alloc().map(|ppn|FrameTracker::new(ppn))
}
pub fn frame_dealloc(ppn:PhysPageNum){
    FRAME_ALLOCATOR.exclusive().dealloc(ppn);
}

