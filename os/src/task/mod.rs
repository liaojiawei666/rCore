
use core::cell::RefCell;

use crate::loader::get_app_data;
use crate::sync::UPSafeCell;
use context::TaskContext;
use crate::trap::TrapContext;
use lazy_static::*;
use task::{TaskControlBlock, TaskStatus};
pub mod context;
pub mod switch;
use crate::loader::get_num_app;
use alloc::vec::Vec;
use self::switch::__switch;
pub mod task;

pub struct TaskManager {
    num_app: usize,//任务的数量
    inner: RefCell<TaskManagerInner>,//所有任务的状态
}
unsafe impl Sync for TaskManager {}

struct TaskManagerInner {
    current_task: usize,//当前正在运行的任务
    tasks: Vec<TaskControlBlock>
}



lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        println!("TaskManager init!");
        let num_app = get_num_app();
        println!("num_app: {}", num_app);
        let mut tasks :Vec<TaskControlBlock>= Vec::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(get_app_data(i),i));
        }
        TaskManager {
            num_app,
            inner: RefCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                }),
        }
    };
}


pub fn suspend_current_and_run_next(){
    TASK_MANAGER.mark_current_suspend();
    run_next_task();
}
pub fn exit_current_and_run_next(){
    TASK_MANAGER.mark_current_exit();
    run_next_task();
}

fn run_next_task(){
    TASK_MANAGER.run_next_task();
}
impl TaskManager{
    fn mark_current_suspend(&self){
        let mut inner=self.inner.borrow_mut();
        let current_task=inner.current_task;
        inner.tasks[current_task].task_status=TaskStatus::Ready;
    }
    fn mark_current_exit(&self){
        let mut inner=self.inner.borrow_mut();
        let current_task=inner.current_task;
        inner.tasks[current_task].task_status=TaskStatus::Exited;
    }
    fn run_next_task(&self){
        if let Some(next)=self.find_next_task(){
            let mut inner=self.inner.borrow_mut();
            let current=inner.current_task;
            inner.current_task=next;
            inner.tasks[next].task_status=TaskStatus::Running;
            let current_task_cx_ptr=&mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr=&inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            unsafe{
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        }else{
            panic!("All applictions completed!")
        }
    }
    fn find_next_task(&self)->Option<usize>{
        let inner=self.inner.borrow();
        let mut next_task=inner.current_task;
        for _ in 0..self.num_app{
            next_task=(next_task+1)%self.num_app;
            if inner.tasks[next_task].task_status==TaskStatus::Ready{
                return Some(next_task);
            }
        }
        None
    }
    fn run_first_task(&self)->!{
        let mut inner=self.inner.borrow_mut();
        let task0=&mut inner.tasks[0];
        task0.task_status=TaskStatus::Running;
        let next_task_cx_ptr: *const TaskContext=&task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused=TaskContext::zero_init();
        unsafe{
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }
    fn get_current_token(&self)->usize{
        let inner=self.inner.borrow();
        let current_task=inner.current_task;
        inner.tasks[current_task].get_user_token()
    }
    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
    }


}
pub fn run_first_task()->!{
    TASK_MANAGER.run_first_task();
}
pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}
pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}