use super::context::TaskContext;
use crate::trap::TrapContext;
use crate::mm::{PhysPageNum,MemorySet,KERNEL_SPACE,VirtAddr,MapPermission};
use crate::trap::trap_handler;
use crate::config::*;
#[derive(Copy,Clone,PartialEq)]
pub enum TaskStatus{
    UnInit,
    Ready,
    Running,
    Exited,
}

pub struct TaskControlBlock{
    pub task_status:TaskStatus,//任务的状态
    pub task_cx:TaskContext,//任务的上下文
    pub memory_set:MemorySet,//任务的内存空间
    pub trap_cx_ppn:PhysPageNum,//任务的陷入上下文的页号    
    pub base_size:usize,//任务的大小
}

impl TaskControlBlock{
    pub fn new(elf_data:&[u8],app_id:usize)->Self{
         // memory_set with elf program headers/trampoline/trap context/user stack
         let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
         let trap_cx_ppn = memory_set
             .translate(VirtAddr::from(TRAP_CONTEXT).into())
             .unwrap()
             .ppn();
         let task_status = TaskStatus::Ready;
         // map a kernel-stack in kernel space
         let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
         KERNEL_SPACE
             .exclusive()
             .insert_framed_area(
                 kernel_stack_bottom.into(),
                 kernel_stack_top.into(),
                 MapPermission::R | MapPermission::W,
             );
         let task_control_block = Self {
             task_status,
             task_cx: TaskContext::goto_trap_return(kernel_stack_top),
             memory_set,
             trap_cx_ppn,
             base_size: user_sp,
         };
         // prepare TrapContext in user space
         let trap_cx = task_control_block.get_trap_cx();
         *trap_cx = TrapContext::app_init_context(
             entry_point,
             user_sp,
             KERNEL_SPACE.exclusive().token(),
             kernel_stack_top,
             trap_handler as usize,
         );
         task_control_block
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    
}