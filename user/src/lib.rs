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

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start()->!{
    clear_bss();
    exit(main());
    panic!("unreachable after exit!");
}

fn clear_bss(){
    extern "C"{
        fn sbss();
        fn ebss();
    }
    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe {
            (a as *mut u8).write_volatile(0);
        }
    });
}

#[linkage = "weak"]
#[no_mangle]
fn main()->i32{
    0
}