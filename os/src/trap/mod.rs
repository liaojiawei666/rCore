use core::arch::global_asm;
use core::arch::asm;
use riscv::register::{
    scause::{self, Exception, Trap,Interrupt},
    stval,
    sie,
    stvec::{self, TrapMode},
};
global_asm!(include_str!("trap.S"));
pub mod context;
use crate::task::processor::current_trap_cx;
use crate::{syscall::syscall, task::{exit_current_and_run_next,suspend_current_and_run_next}};
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::task::processor::current_user_token;
pub use context::TrapContext;
pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry(){
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry(){
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}
#[no_mangle]
pub fn trap_from_kernel()->!{
    panic!("a trap from kernel!");
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let cx=current_trap_cx();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            let mut cx=current_trap_cx();
            cx.sepc += 4;
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
            cx=current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!("[kernel] PageFault in application, bad addr = {:#x}, bad instruction = {:#x}, kernel killed it.", stval, cx.sepc);
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            crate::timer::set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}
#[no_mangle]
pub fn trap_return() -> ! {
    set_user_trap_entry();
    let trap_cx_ptr=TRAP_CONTEXT;
    let user_satp=current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va=__restore as usize - __alltraps as usize+TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr     {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}