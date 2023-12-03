use core::cell::RefMut;
use alloc::sync::{Arc,Weak};
use alloc::vec::Vec;

use super::context::TaskContext;
use super::pid::{PidHandle, KernelStack,pid_alloc};
use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use crate::mm::{PhysPageNum,MemorySet,KERNEL_SPACE,VirtAddr};
use crate::trap::trap_handler;
use crate::config::*;
#[derive(Copy,Clone,PartialEq)]
pub enum TaskStatus{
    UnInit,
    Ready,
    Running,
    Zombie,
    Exited,
}

pub struct TaskControlBlock{
    pub pid:PidHandle,
    pub kernel_stack:KernelStack,
    inner:UPSafeCell<TaskControlBlockInner>
}
pub struct TaskControlBlockInner{
    pub trap_cx_ppn:PhysPageNum,
    pub base_size:usize,
    pub task_cx:TaskContext,
    pub task_status:TaskStatus,
    pub memory_set:MemorySet,
    pub parent:Option<Weak<TaskControlBlock>>,
    pub children:Vec<Arc<TaskControlBlock>>,
    pub exit_code:i32,
}

impl TaskControlBlockInner{
    pub fn get_trap_cx(&self)->&'static mut TrapContext{
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self)->usize{
        self.memory_set.token()
    }
    pub fn get_status(&self)->TaskStatus{
        self.task_status
    }
    pub fn is_zombie(&self)->bool{
        self.get_status()==TaskStatus::Zombie
    }
}

impl TaskControlBlock{
    pub fn new(elf_data:&[u8])->Self{
        let (memory_set,user_sp,entry_point)=MemorySet::from_elf(elf_data);
        let trap_cx_ppn=memory_set
        .translate(VirtAddr::from(TRAP_CONTEXT).into())
        .unwrap()
        .ppn();
        let pid_handle=pid_alloc();
        let kernel_stack=KernelStack::new(&pid_handle);
        let kernel_stack_top=kernel_stack.get_top();
        let task_control_block=Self{
            pid:pid_handle,
            kernel_stack,
            inner:unsafe {
                UPSafeCell::new(TaskControlBlockInner{
                    trap_cx_ppn,
                    base_size:user_sp,
                    task_cx:TaskContext::goto_trap_return(kernel_stack_top),
                    task_status:TaskStatus::Ready,
                    memory_set,
                    parent:None,
                    children:Vec::new(),
                    exit_code:0,
                })
            }
        };
        let trap_cx= task_control_block.inner_exclusive().get_trap_cx();
        *trap_cx=TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive().token(),
        kernel_stack_top,
            trap_handler as usize);
        task_control_block
    }
    pub fn inner_exclusive(&self)->RefMut<'_,TaskControlBlockInner>{
        self.inner.exclusive()
    }
    pub fn getpid(&self)->usize{
        self.pid.0
    }
    pub fn exec(&self,elf_data:&[u8]){
        let (memory_set,user_sp,entry_point)=MemorySet::from_elf(elf_data);
        let trap_cx_ppn=memory_set.translate(VirtAddr::from(TRAP_CONTEXT).into()).unwrap().ppn();
        let mut inner=self.inner_exclusive();
        inner.memory_set=memory_set;
        inner.trap_cx_ppn=trap_cx_ppn;
        let trap_cx=inner.get_trap_cx();
        *trap_cx=TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive().token(),
            self.kernel_stack.get_top(),
            trap_handler as usize
        );
    }
    pub fn fork(self:&Arc<TaskControlBlock>)->Arc<TaskControlBlock>{
        let mut parent_inner=self.inner_exclusive();
        let memory_set=MemorySet::from_existed_user(&parent_inner.memory_set);
        let trap_cx_ppn=memory_set.translate(
            VirtAddr::from(TRAP_CONTEXT).into()
        ).unwrap().ppn();
        let pid_handle=pid_alloc();
        let kernel_stack=KernelStack::new(&pid_handle);
        let kernel_stack_top=kernel_stack.get_top();
        let task_control_block=Arc::new(TaskControlBlock{
            pid:pid_handle,
            kernel_stack,
            inner:unsafe {
                UPSafeCell::new(TaskControlBlockInner{
                    trap_cx_ppn,
                    base_size:parent_inner.base_size,
                    task_cx:TaskContext::goto_trap_return(kernel_stack_top),
                    task_status:TaskStatus::Ready,
                    memory_set,
                    parent:Some(Arc::downgrade(self)),
                    children:Vec::new(),
                    exit_code:0,
                })
            }
        });
        parent_inner.children.push(Arc::clone(&task_control_block));
        let trap_cx=task_control_block.inner_exclusive().get_trap_cx();
        trap_cx.kernel_sp=kernel_stack_top;
        task_control_block
    }
    
}