use core::arch::global_asm;
use crate::batch::run_next_app;
use riscv::register::{stvec::{self,TrapMode}, scause::{self, Trap,Exception}, stval};
global_asm!(include_str!("trap.asm"));
pub mod context;
pub use context::TrapContext;
use crate::syscall::syscall;
pub fn init(){
// 在 RV64 中， stvec 是一个 64 位的 CSR，在中断使能的情况下，保存了中断处理的入口地址。它有两个字段：
//  MODE 位于 [1:0]，长度为 2 bits；
//  BASE 位于 [63:2]，长度为 62 bits。
//  当 MODE 字段为 0 的时候， stvec 被设置为 Direct 模式，此时进入 S 模式的 Trap 无论原因如何，处理 Trap 的入口地址都是 BASE<<2 ， CPU 会跳转到这个地方进行异常处理。
    extern "C"{ fn __alltraps();}
    unsafe{
        stvec::write(__alltraps as usize,TrapMode::Direct);
    }
}

#[no_mangle]
pub fn trap_handler(cx:&mut TrapContext)->&mut TrapContext{
    let scause = scause::read();
    let stval=stval::read();
    match scause.cause(){
        Trap::Exception(Exception::UserEnvCall)=>{
            cx.sepc+=4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        Trap::Exception(Exception::StoreFault)|Trap::Exception(Exception::StorePageFault)=>{
            println!("[kernel] PageFault in application, kernel killed it.");
            run_next_app();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            run_next_app();
        }
        _ => {
            panic!("Unsupported trap {:?}, stval = {:#x}!", scause.cause(), stval);
        }
    }
    cx
}