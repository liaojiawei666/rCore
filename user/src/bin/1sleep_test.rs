#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;


#[no_mangle]
unsafe fn main() -> i32 {
    println!("task01 before sleep 2s");
    user_lib::sleep(2000);
    println!("task01 after sleep 2s");
    0
}
