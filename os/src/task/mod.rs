mod action;
mod context;
mod manager;
mod pid;
pub mod processor;
mod signal;
mod switch;
pub use signal::SignalFlags;
pub mod task;
use crate::fs::{open_file, OpenFlags};
use crate::task::manager::remove_from_pid2task;
pub use action::{SignalAction, SignalActions};
use alloc::sync::Arc;
use lazy_static::lazy_static;
pub use manager::{add_task, pid2task};
pub use processor::run_tasks;
pub use signal::MAX_SIG;
pub const IDLE_PID: usize = 0;

use self::{
    context::TaskContext,
    processor::{current_task, schedule, take_current_task},
    task::{TaskControlBlock, TaskStatus},
};
lazy_static! {
    pub static ref INITPROC: Arc<TaskControlBlock> = Arc::new({
        let inode = open_file("initproc", OpenFlags::RDONLY).unwrap();
        let v = inode.read_all();
        TaskControlBlock::new(v.as_slice())
    });
}
pub fn add_initproc() {
    add_task(INITPROC.clone());
    println!("add_initproc!");
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
    remove_from_pid2task(task.getpid());
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
pub fn current_add_signal(signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive();
    task_inner.signals |= signal;
}
pub fn check_signals_error_of_current() -> Option<(i32, &'static str)> {
    let task = current_task().unwrap();
    let task_inner = task.inner_exclusive();
    task_inner.signals.check_error()
}

fn check_pending_signals() {
    for sig in 0..(MAX_SIG + 1) {
        let task = current_task().unwrap();
        let task_inner = task.inner_exclusive();
        let signal = SignalFlags::from_bits(1 << sig).unwrap();
        if task_inner.signals.contains(signal) && (!task_inner.signal_mask.contains(signal)) {
            let mut masked = true;//是否屏蔽信号signal,true表示屏蔽,false表示不屏蔽
            let handling_sig = task_inner.handling_sig;
            if handling_sig == -1 {//没有正在处理的信号
                masked = false;
            } else {//有正在处理的信号，但是处理handling_sig的时没有屏蔽signal信号
                let handling_sig = handling_sig as usize;
                if !task_inner.signal_actions.table[handling_sig]
                    .mask
                    .contains(signal)
                {
                    masked = false;
                }
            }
            if !masked {//如果没有屏蔽信号signal，则处理信号signal
                drop(task_inner);
                drop(task);
                if signal == SignalFlags::SIGKILL
                    || signal == SignalFlags::SIGSTOP
                    || signal == SignalFlags::SIGCONT
                    || signal == SignalFlags::SIGDEF
                {//杀死、暂停、继续、默认信号由内核处理
                    call_kernel_signal_handler(signal);
                } else {//其他信号由用户处理
                    call_user_signal_handler(sig, signal);
                    return;
                }
            }
        }
    }
}

fn call_kernel_signal_handler(signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive();
    match signal {
        SignalFlags::SIGSTOP => {//frozen为true代表当前进程暂停，处理结束后使用异或清除暂停信号
            task_inner.frozen = true;
            task_inner.signals ^= SignalFlags::SIGSTOP;
        }
        SignalFlags::SIGCONT => {//这里需要if判断一下，因为如果进程没有被暂停也可能收到SIGCONT信号
            if task_inner.signals.contains(SignalFlags::SIGCONT) {
                task_inner.signals ^= SignalFlags::SIGCONT;
                task_inner.frozen = false;
            }
        }
        _ => {
            task_inner.killed = true;
        }
    }
}

fn call_user_signal_handler(sig: usize, signal: SignalFlags) {
    let task = current_task().unwrap();
    let mut task_inner = task.inner_exclusive();

    let handler = task_inner.signal_actions.table[sig].handler;
    if handler != 0 {
        // user handler

        // handle flag
        task_inner.handling_sig = sig as isize;
        task_inner.signals ^= signal;

        // backup trapframe
        let  trap_ctx = task_inner.get_trap_cx();
        task_inner.trap_ctx_backup = Some(*trap_ctx);

        // modify trapframe
        trap_ctx.sepc = handler;

        // put args (a0)
        trap_ctx.x[10] = sig;
    } else {
        // default action
        println!("[K] task/call_user_signal_handler: default action: ignore it or kill process");
    }
}

pub fn handle_signals() {
    loop {
        check_pending_signals();//处理当前进程所有处于pending状态的信号
        let (frozen, killed) = {
            let task = current_task().unwrap();
            let task_inner = task.inner_exclusive();
            (task_inner.frozen, task_inner.killed)
        };
        //如果进程正常运行或者被杀死了，信号处理完毕就可以退出循环了，进行后续处理
        if !frozen || killed {
            break;
        }
        //如果进程被暂停了，则挂起当前进程并运行下一个进程
        suspend_current_and_run_next();
    }
}
