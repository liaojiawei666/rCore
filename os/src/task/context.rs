use crate::trap::trap_return;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TaskContext {
    ra: usize,//return address 返回地址
    sp:usize,//stack pointer 栈指针
    s: [usize; 12],//s registers s寄存器
}

impl TaskContext {
    pub fn zero_init() -> Self {
        TaskContext {
            ra: 0,
            sp:0,
            s: [0; 12],
        }
    }
    pub fn goto_trap_return(kstack_ptr:usize)->Self{
       Self { ra: trap_return as usize, s: [0;12],sp:kstack_ptr }
    }
}


