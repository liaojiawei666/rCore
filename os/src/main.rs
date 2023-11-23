//! The main module and entrypoint
//!
//! The operating system and app also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality [`clear_bss()`]. (See its source code for
//! details.)
//!
//! We then call [`println!`] to display `Hello, world!`.


#![no_std]
#![no_main]
#![feature(panic_info_message)]
mod sbi;
#[macro_use]
mod console;
mod logging;
mod lang_items;
mod trap;
mod batch;
mod sync;
use log::*;
mod syscall;
use core::arch::global_asm;
global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));
#[no_mangle]
fn rust_main() -> ! {
    extern "C"{
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack_lower_bound();
        fn boot_stack_top();
    }
    
    clear_bss();
    logging::init();
    println!("Hello, world!");
    trace!("[kernel] .text [{:#x}, {:#x})", stext as usize, etext as usize);
    debug!("[kernel] .rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
    info!("[kernel] .data [{:#x}, {:#x})", sdata as usize, edata as usize);
    warn!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    error!("[kernel] boot_stack [{:#x}, {:#x})", boot_stack_lower_bound as usize, boot_stack_top as usize);
    trap::init();
    batch::init();
    batch::run_next_app();
    
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