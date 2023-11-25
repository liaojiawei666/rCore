use crate::sync::UPSafeCell;
use context::TaskContext;
use lazy_static::*;
use task::{TaskControlBlock, TaskStatus};
use crate::loader::init_app_cx;
pub mod context;
pub mod switch;
use crate::loader::get_num_app;
use crate::config::*;

use self::switch::__switch;
pub mod task;

pub struct TaskManager {
    num_app: usize,//任务的数量
    inner: UPSafeCell<TaskManagerInner>,//所有任务的状态
}

struct TaskManagerInner {
    current_task: usize,//当前正在运行的任务
    tasks: [TaskControlBlock; MAX_APP_NUM],//每个元素对应一个任务的状态
}



lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [TaskControlBlock {
            task_cx: TaskContext::zero_init(),
            task_status: TaskStatus::UnInit,
        }; MAX_APP_NUM];
        for i in 0..num_app {
            tasks[i].task_cx = TaskContext::goto_restore(init_app_cx(i));
            tasks[i].task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
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

pub fn run_next_task(){
    TASK_MANAGER.run_next_task();
}
impl TaskManager{
    fn mark_current_suspend(&self){
        let mut inner=self.inner.exclusive();
        let current_task=inner.current_task;
        inner.tasks[current_task].task_status=TaskStatus::Ready;
    }
    fn mark_current_exit(&self){
        let mut inner=self.inner.exclusive();
        let current_task=inner.current_task;
        inner.tasks[current_task].task_status=TaskStatus::Exited;
    }
    fn run_next_task(&self){
        if let Some(next)=self.find_next_task(){
            let mut inner=self.inner.exclusive();
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
        let inner=self.inner.exclusive();
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
        let mut inner=self.inner.exclusive();
        let task0=&mut inner.tasks[0];
        task0.task_status=TaskStatus::Running;
        let next_task_cx_ptr=&task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused=TaskContext::zero_init();
        unsafe{
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }
}
pub fn run_first_task()->!{
    TASK_MANAGER.run_first_task();
}
