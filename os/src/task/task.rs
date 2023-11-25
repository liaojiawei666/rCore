use super::context::TaskContext;
#[derive(Copy,Clone,PartialEq)]
pub enum TaskStatus{
    UnInit,
    Ready,
    Running,
    Exited,
}
#[derive(Copy,Clone)]
pub struct TaskControlBlock{
    pub task_status:TaskStatus,//任务的状态
    pub task_cx:TaskContext,//任务的上下文
}