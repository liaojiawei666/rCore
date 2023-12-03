#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(linkage)]
#![no_std]

pub mod syscall;
use syscall::*;
#[macro_use]
pub mod console;
pub mod lang_items;

use buddy_system_allocator::LockedHeap;
const USER_HEAP_SIZE:usize=16384;
static mut HEAP_SPACE:[u8;USER_HEAP_SIZE]= [0;USER_HEAP_SIZE];
#[global_allocator]
static HEAP:LockedHeap = LockedHeap::empty();

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}


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
pub fn read(fd:usize,buffer:&mut [u8])->isize{
    sys_read(fd,buffer)
}
pub fn getpid()->isize{
    sys_getpid()
}
pub fn fork()->isize{
    sys_fork()
}
pub fn exec(path:&str)->isize{
    sys_exec(path)
}
pub fn waitpid(pid:usize,exit_code:&mut i32)->isize{
    loop{
        match sys_waitpid(pid as isize, exit_code as *mut _){
            -2=>{
                yield_();
            }
            exit_pid=>return exit_pid,
        }
    }
}
pub fn wait(exit_code: &mut i32) -> isize {
    loop {
        match sys_waitpid(-1, exit_code as *mut _) {
            -2 => {
                yield_();
            }
            // -1 or a real pid
            exit_pid => return exit_pid,
        }
    }
}



#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start()->!{
    unsafe{
        HEAP.lock().init(HEAP_SPACE.as_ptr() as usize,USER_HEAP_SIZE);
    }
    exit(main());
    panic!("unreachable after exit!");
}


#[linkage = "weak"]
#[no_mangle]
fn main()->i32{
    0
}