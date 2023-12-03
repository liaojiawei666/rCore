use crate::loader::get_app_data_by_name;
use crate::task::{suspend_current_and_run_next,exit_current_and_run_next};
use crate::task::processor::{current_task, current_user_token};
use crate::task::add_task;
use crate::mm::{translated_str, translated_refmut};
use crate::timer::get_time_ms; 
use alloc::sync::Arc;
pub fn sys_yield()->isize{
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}
pub fn sys_fork()->isize{
    let current_task=current_task().unwrap();
    let new_task=current_task.fork();
    let new_pid=new_task.pid.0;
    let trap_cx=new_task.inner_exclusive().get_trap_cx();
    trap_cx.x[10]=0;
    add_task(new_task);
    new_pid as isize
}
pub fn sys_exec(path:*const u8)->isize{
    let token=current_user_token();
    let path=translated_str(token, path);
    if let Some(data)=get_app_data_by_name(path.as_str()){
        let task=current_task().unwrap();
        task.exec(data);
        0
    }else{
        -1
    }
}
pub fn sys_exit(exit_code:i32)->!{
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}
//-1: no child,-2 子进程不处于僵尸状态 other:子进程id
pub fn sys_waitpid(pid:isize,exit_code_ptr:*mut i32)->isize{
    let task=current_task().unwrap();
    let mut inner: core::cell::RefMut<'_, crate::task::task::TaskControlBlockInner>=task.inner_exclusive();
    if inner.children
    .iter()
    .find(|p|{pid==-1||pid as usize==p.getpid()})
    .is_none(){
        return -1;
    }
    let pair=inner.children
    .iter()
    .enumerate()
    .find(|(_,p)|{
        p.inner_exclusive().is_zombie()&&(pid==-1||pid as usize==p.getpid())
    });
    if let Some((idx,_))=pair{
        let child=inner.children.remove(idx);
        assert_eq!(Arc::strong_count(&child),1,"Only initproc can wait child!");
        let found_pid=child.getpid();
        let exit_code=child.inner_exclusive().exit_code;
        *translated_refmut(inner.memory_set.token(),exit_code_ptr)=exit_code;
        found_pid as isize
    }else{
        -2
    }
}
pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}
