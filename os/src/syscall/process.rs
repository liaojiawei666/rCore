use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_ref, translated_refmut, translated_str};
use crate::task::processor::{current_task, current_user_token};
use crate::task::SignalAction;
use crate::task::SignalFlags;
use crate::task::MAX_SIG;
use crate::task::{add_task, pid2task};
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next};
use crate::timer::get_time_ms;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_get_time() -> isize {
    get_time_ms() as isize
}
pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    let trap_cx = new_task.inner_exclusive().get_trap_cx();
    trap_cx.x[10] = 0;
    add_task(new_task);
    new_pid as isize
}
pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    println!("{:?}:{:#x}", path, args as usize);
    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe {
            args = args.add(1);
        }
    }
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice(), args_vec);
        0
    } else {
        -1
    }
}
pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}
//-1: no child,-2 子进程不处于僵尸状态 other:子进程id
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    let mut inner: core::cell::RefMut<'_, crate::task::task::TaskControlBlockInner> =
        task.inner_exclusive();
    if inner
        .children
        .iter()
        .find(|p| pid == -1 || pid as usize == p.getpid())
        .is_none()
    {
        return -1;
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        p.inner_exclusive().is_zombie() && (pid == -1 || pid as usize == p.getpid())
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        assert_eq!(
            Arc::strong_count(&child),
            1,
            "Only initproc can wait child!"
        );
        let found_pid = child.getpid();
        let exit_code = child.inner_exclusive().exit_code;
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
}
pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}
pub fn sys_sigprocmask(mask: u32) -> isize {
    if let Some(task) = current_task() {
        let mut inner = task.inner_exclusive();
        let old_mask = inner.signal_mask;
        if let Some(flag) = SignalFlags::from_bits(mask) {
            inner.signal_mask = flag;
            old_mask.bits() as isize
        } else {
            -1
        }
    } else {
        -1
    }
}
fn check_sigaction_error(signal: SignalFlags, action: usize, old_action: usize) -> bool {
    if action == 0
        || old_action == 0
        || signal == SignalFlags::SIGKILL
        || signal == SignalFlags::SIGSTOP
    {
        true
    } else {
        false
    }
}

pub fn sys_sigaction(
    signum: i32,
    action: *const SignalAction,
    old_action: *mut SignalAction,
) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive();
    if signum as usize > MAX_SIG {
        return -1;
    }
    if let Some(flag) = SignalFlags::from_bits(1 << signum) {
        if check_sigaction_error(flag, action as usize, old_action as usize) {
            return -1;
        }
        let prev_action = inner.signal_actions.table[signum as usize];
        *translated_refmut(token, old_action) = prev_action;
        inner.signal_actions.table[signum as usize] = *translated_ref(token, action);
        0
    } else {
        -1
    }
}
pub fn sys_kill(pid: usize, signum: i32) -> isize {
    if let Some(task) = pid2task(pid) {
        if let Some(flag) = SignalFlags::from_bits(1 << signum) {
            let mut task_ref = task.inner_exclusive();
            if task_ref.signals.contains(flag) {
                //对于同一个信号，在上一个没处理完的情况下，不再处理
                return -1;
            }
            task_ref.signals.insert(flag); //插入一个信号，后续由handle_signals处理
            0
        } else {
            -1
        }
    } else {
        -1
    }
}
pub fn sys_sigreturn() -> isize {
    if let Some(task) = current_task() {
        let mut inner = task.inner_exclusive();
        inner.handling_sig = -1;
        let trap_ctx = inner.get_trap_cx();
        *trap_ctx = inner.trap_ctx_backup.unwrap();
        trap_ctx.x[10] as isize
    } else {
        -1
    }
}
