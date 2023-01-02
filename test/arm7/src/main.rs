#![no_std]
#![no_main]
extern crate alloc;

use dsengine_arm7;

#[no_mangle]
extern "C" fn main() -> ! {
    dsengine_arm7::main_loop();
}
