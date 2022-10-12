#![no_std]
#![no_main]
#![feature(isa_attribute)]
extern crate alloc;
use core::ptr;
use core::fmt::Write;
use ironds as nds;
//use ironds::nocash;
//use alloc::string::String;

const DISPCNT: u32 = 0x04000000;
const VRAMCNT_A: u32 = 0x04000240;

use arrayvec::ArrayString;

static mut framectr: u32 = 0;
static mut ctr2: u32 = 0;

#[no_mangle]
extern "C" fn main() -> ! {
    unsafe {
        ptr::write_volatile(DISPCNT as *mut u32, 2 << 16);
        ptr::write_volatile(VRAMCNT_A as *mut u8, 0b1000_0000);
        ptr::write_volatile((0x06800000 + 256 * 4) as *mut u16, 0xFFFF);
    }
    nds::interrupt::irq_set_handler(Some(inter));
    nds::interrupt::irq_enable(nds::interrupt::IRQFlags::VBLANK | nds::interrupt::IRQFlags::VCOUNT);
    nds::display::set_vcount_trigger(100);

    let mut stuff = ArrayString::<100>::new();

    nds::timers::start_profiler_timer(0);
    sieve_bench();
    let time = nds::timers::end_profiler_timer(0);
    write!(&mut stuff, "{}\n", time).unwrap();

    nds::timers::start_profiler_timer(0);
    sieve_bench();
    let time = nds::timers::end_profiler_timer(0);
    write!(&mut stuff, "{}\n", time).unwrap();

    nds::timers::start_profiler_timer(0);
    unsafe {core::ptr::copy(0x02000000 as *const u8, 0x02200000 as *mut u8, 1000); }
    let time = nds::timers::end_profiler_timer(0);
    write!(&mut stuff, "{}\n", time).unwrap();

    stuff.push_str("hoofa");
    stuff.push_str(" toofa\n");
    //nocash::print(stuff.as_str());
    //nocash::print2(stuff.as_str());
    //nds::display::power_off(nds::display::GfxPwr::ALL);
    //nds::display::set_engine_lcd(nds::display::MainEnginePos::BOTTOM);
    //nds::display::map_vram_block_a(nds::display::vram_type::A::LCDC);
    let mut _v: alloc::vec::Vec<u8> = alloc::vec![1; 10];//alloc::vec::Vec::with_capacity_exact(0x10000000);
    let mut _v2: alloc::vec::Vec<u8>;
    {
    let mut _v1: alloc::vec::Vec<u8> = alloc::vec![1; 20];
    _v2 = alloc::vec![1; 30];

    write!(&mut stuff, "{:p}\n", _v.as_ptr()).unwrap();
    write!(&mut stuff, "{:p}\n", _v1.as_ptr()).unwrap();
    write!(&mut stuff, "{:p}\n\n", _v2.as_ptr()).unwrap();
    }
    let mut _v3: alloc::vec::Vec<u8> = alloc::vec![0; 21];
    write!(&mut stuff, "{:p}\n", _v3.as_ptr()).unwrap();

    //panic!("stuff");
    nds::display::console::init_default();
    nds::display::console::print("Hello from Rust on the DS!\n\n");
    nds::display::console::print(stuff.as_str());
    //nds::display::set_main_display_control(nds::display::get_main_display_control()
    //    .with_bg_mode(nds::display::bgmode::b)
    //    .with_bg0_3d(false)
    //    .with_tile_obj(nds::display::tile_obj_2D));
    let mut counter: u32 = 0;
    loop {
        let mut stuff = ArrayString::<50>::new();
        write!(&mut stuff, "{}\n", counter).unwrap();
        counter = counter.wrapping_add(1);
        write!(&mut stuff, "{}\n", unsafe {framectr} / 60 ).unwrap();
        write!(&mut stuff, "{}\n", unsafe {ctr2} / 60 ).unwrap();
        write!(&mut stuff, "{}\n", nds::mmio::VCOUNT.read()).unwrap();
        write!(&mut stuff, "{}\n", nds::input::read_keys().bits()).unwrap();
        //write!(&mut stuff, "{}\n", unsafe{ptr::read_volatile(0x023FF008 as *const u32)}).unwrap();
        //write!(&mut stuff, "{}\n", unsafe{nds::mmio::DISPSTAT.read() as u32}).unwrap();
        nds::display::console::set_cursor_pos(0, 15);
        nds::display::console::print(stuff.as_str());
        nds::interrupt::wait_for_vblank();
    }
}

// https://en.wikipedia.org/wiki/Byte_Sieve
//#[link_section = ".itcm"]
#[inline(never)]
fn sieve_bench() -> u32 {
    //use core::arch::asm;
    //let mut x: u32;
    /*unsafe {asm!(
        "mov r11, r11",
        /*"mov {tmp}, r15",
        tmp = out(reg) x,*/
    );}*/
    //nocash::breakpoint!();
    //let mut prn = ArrayString::<20>::new();
    //write!(&mut prn, "{}\n", x).unwrap();
    //nocash::print(prn.as_str());

    const SIZE: usize = 8191;
    let mut flags: [u8; SIZE] = [0; SIZE];
    let mut count: u32 = 0;
    for _iter in 0..10 {
        for i in 0..SIZE {
            flags[i] = 1;
        }
        for i in 0..SIZE {
            if flags[i] == 1 {
                let prime: u32 = (i as u32 * 2) + 3;
                let mut k = i + prime as usize;
                while k < SIZE {
                    flags[k] = 0;
                    k += prime as usize;
                }
                count += 1;
            }
        }
    }
    count
}

extern "C" fn inter (f: nds::interrupt::IRQFlags) {
    if f.contains(nds::interrupt::IRQFlags::VBLANK) {
        unsafe { framectr += 1; }
    }
    else if f.contains(nds::interrupt::IRQFlags::VCOUNT) {
        unsafe {
            ctr2 += 1;
            if ctr2 & 1 > 0 {
                nds::display::set_vcount_trigger(100);
            } else {
                nds::display::set_vcount_trigger(130);
            }
        }
    }
}
