use riscv::register::time;
use crate::config::CLOCK_FREQ;
use crate::sbi::set_timer;
const TICKS_PER_SEC:usize=100;
const MICRO_PER_SEC:usize=1_000_000;
pub fn set_next_trigger(){
    set_timer(get_time()+CLOCK_FREQ/TICKS_PER_SEC);
}
pub fn get_time()->usize{
    time::read()
}
pub fn get_time_us()->usize{
    time::read()*MICRO_PER_SEC/CLOCK_FREQ
}
#[repr(C)]
pub struct TimeVal{
    pub sec:usize,
    pub usec:usize,
}
pub fn sys_get_time(ts:*mut TimeVal,_ts:usize)->isize{
    let us=get_time_us();
    unsafe{
        *ts=TimeVal{
            sec:us/MICRO_PER_SEC,
            usec:us%MICRO_PER_SEC,
        };
    }
    0
}