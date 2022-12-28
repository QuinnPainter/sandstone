#![no_std]
#![no_main]
extern crate alloc;
use ironds as nds;

#[no_mangle]
extern "C" fn main() -> ! {
    nds::interrupt::irq_enable(nds::interrupt::IRQFlags::VBLANK);

    loop {
        nds::input::scan_keys();
        nds::interrupt::wait_for_vblank();
    }
}
