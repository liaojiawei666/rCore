mod context;
mod manager;
mod pid;
pub mod processor;
mod switch;
pub mod task;
use crate::loader::get_app_data_by_name;
use alloc::sync::Arc;
use lazy_static::lazy_static;
pub use manager::add_task;
pub use processor::run_tasks;
pub const IDLE_PID: usize = 0;


use self::{
    context::TaskContext,
    processor::{schedule, take_current_task},
    task::{TaskControlBlock, TaskStatus},
};
lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new(TaskControlBlock::new(
        get_app_data_by_name("initproc").unwrap()
    ));
}
pub fn add_initproc() {
    add_task(INITPROC.clone());
}
pub fn suspend_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_exclusive();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    add_task(task);
    schedule(task_cx_ptr);
}
pub fn exit_current_and_run_next(exit_code: i32) {
    // take from Processor
    let task = take_current_task().unwrap();

    // **** access current TCB exclusively
    let mut inner = task.inner_exclusive();
    // Change status to Zombie
    inner.task_status = TaskStatus::Zombie;
    // Record exit code
    inner.exit_code = exit_code;
    // do not move to its parent but under initproc

    // ++++++ access initproc TCB exclusively
    {
        let mut initproc_inner = INITPROC.inner_exclusive();
        for child in inner.children.iter() {
            child.inner_exclusive().parent = Some(Arc::downgrade(&INITPROC));
            initproc_inner.children.push(child.clone());
        }
    }
    // ++++++ release parent PCB

    inner.children.clear();
    // deallocate user space
    inner.memory_set.recycle_data_pages();
    drop(inner);
    // **** release current PCB
    // drop task manually to maintain rc correctly
    drop(task);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    schedule(&mut _unused as *mut _);
}
