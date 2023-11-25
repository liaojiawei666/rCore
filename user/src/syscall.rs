use core::{arch::asm, borrow::BorrowMut};
fn syscall(id:usize,args:[usize;3])->isize{
    let mut ret:isize;
    unsafe{
        asm!(
            "ecall",
            inlateout("x10") args[0]=>ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id,
        );
    }
    ret
}

const SYSCALL_WRITE:usize = 64;
const SYSCALL_EXIT:usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GETTIME: usize = 169;

pub fn sys_write(fd:usize,buffer:&[u8])->isize{
    syscall(SYSCALL_WRITE,[fd,buffer.as_ptr() as usize,buffer.len()])
}
pub fn sys_exit(xstate:i32)->isize{
    syscall(SYSCALL_EXIT,[xstate as usize,0,0])
}
pub fn sys_yield()->isize{
    syscall(SYSCALL_YIELD,[0,0,0])
}

pub struct TimeVal{
    pub sec:usize,
    pub usec:usize,
}
pub fn sys_get_time()->isize{
    let mut timeval=TimeVal{sec:0,usec:0};
    syscall(SYSCALL_GETTIME,[&mut timeval as *mut TimeVal as usize,0,0]);
    (timeval.sec*1000+timeval.usec/1000) as isize
}