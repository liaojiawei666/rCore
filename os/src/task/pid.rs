use lazy_static::lazy_static;
use alloc::vec::Vec;
use crate::mm::{VirtAddr,KERNEL_SPACE, MapPermission};
use crate::sync::UPSafeCell;
use crate::config::{TRAMPOLINE, KERNEL_STACK_SIZE, PAGE_SIZE};
pub struct PidHandle(pub usize);


struct PidAllocator{
    current:usize,
    recycled:Vec<usize>,
}
impl PidAllocator{
    pub fn new()->Self{
        Self{
            current:0,
            recycled:Vec::new(),
        }
    }
    pub fn alloc(&mut self)->PidHandle{
        if let Some(pid)=self.recycled.pop(){
            PidHandle(pid)
        }else{
            self.current+=1;
            PidHandle(self.current-1)
        }
    }
    pub fn dealloc(&mut self,pid:usize){
        assert!(pid<self.current);
        assert!(
            !self.recycled.contains(&pid),
            "pid {} has been deallocated!", pid
        );
        self.recycled.push(pid);
    }
}
lazy_static!{
    static ref PID_ALLOCATOR:UPSafeCell<PidAllocator>=unsafe{
        UPSafeCell::new(PidAllocator::new())
    };
}

pub fn pid_alloc()->PidHandle{
    PID_ALLOCATOR.exclusive().alloc()
}
impl Drop for PidHandle{
    fn drop(&mut self){
        PID_ALLOCATOR.exclusive().dealloc(self.0);
    }
}

pub struct KernelStack{
    pid:usize,
}

pub fn kernel_stack_position(app_id:usize)->(usize,usize){
    let top=TRAMPOLINE - app_id*(KERNEL_STACK_SIZE+PAGE_SIZE);
    let bottom=top-KERNEL_STACK_SIZE;
    (bottom,top)
}
impl KernelStack{
    pub fn new(pid_handle:&PidHandle)->Self{
        let pid=pid_handle.0;
        let (bottom,top)=kernel_stack_position(pid);
        KERNEL_SPACE.exclusive()
        .insert_framed_area(bottom.into(), top.into(), MapPermission::R|MapPermission::W);
        KernelStack { pid: pid_handle.0 }
    }
    pub fn push_on_top<T>(&self,value:T)->*mut T where T:Sized,{
        let kernel_stack_top=self.get_top();
        let ptr_mut=(kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe{*ptr_mut=value;}
        ptr_mut
    }
    pub fn get_top(&self)->usize{
        let (_,top)=kernel_stack_position(self.pid);
        top
    }
}
impl Drop for KernelStack{
    fn drop(&mut self){
        let (kernel_stack_bottom, _) = kernel_stack_position(self.pid);
        let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
        KERNEL_SPACE
            .exclusive()
            .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
        
    }
}