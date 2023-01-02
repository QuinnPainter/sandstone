#![no_std]
extern crate alloc;
use ironds as nds;

pub fn main_loop() -> ! {
    nds::interrupt::irq_enable(nds::interrupt::IRQFlags::VBLANK);

    loop {
        nds::input::scan_keys();
        nds::interrupt::wait_for_vblank();
    }
}
