#![feature(panic_info_message)]
#![feature(linkage)]
#![no_std]

pub mod syscall;
use syscall::*;
#[macro_use]
pub mod console;
pub mod lang_items;
pub fn write(fd:usize,buffer:&[u8])->isize{
    sys_write(fd,buffer)
}
pub fn exit(exit_code:i32)->isize{
    sys_exit(exit_code)
}

pub fn yield_()->isize{
    sys_yield()
}
pub fn get_time()->isize{
    sys_get_time()
}
pub fn sleep(period_ms: usize) {
    let start = get_time();
    while get_time() < start + period_ms as isize {
        sys_yield();
    }
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start()->!{
    exit(main());
    panic!("unreachable after exit!");
}


#[linkage = "weak"]
#[no_mangle]
fn main()->i32{
    0
}