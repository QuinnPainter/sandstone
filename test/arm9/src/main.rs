#![no_std]
#![no_main]
extern crate alloc;

use dsengine;

#[no_mangle]
extern "C" fn main() -> ! {
    dsengine::main_loop();
}
