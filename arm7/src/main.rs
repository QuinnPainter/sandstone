#![no_std]
#![no_main]
#![feature(isa_attribute)]
extern crate alloc;
//use core::ptr;
use ironds as nds;

#[no_mangle]
extern "C" fn main() -> ! {
    nds::interrupt::irq_enable(nds::interrupt::IRQFlags::VBLANK);

    loop {
        nds::input::scan_keys();
        //unsafe { core::ptr::write_volatile(0x023FF008 as *mut u32, nds::mmio::DISPSTAT.read() as u32); }
        nds::interrupt::wait_for_vblank();
    }
}
