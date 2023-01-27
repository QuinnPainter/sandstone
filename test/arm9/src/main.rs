#![no_std]
#![no_main]
#![allow(unused_imports)]
extern crate alloc;

use alloc::{string::String, rc::Rc, vec::Vec};
use core::{cell::{RefCell, Ref}, any::Any, num::NonZeroU32};
use dsengine::{hierarchy::{self, Node}, Script, ScriptContext};
use dsengine::pool::Pool;
use ironds::nocash;
use ironds::{display::console, sync::{NdsCell, NdsMutex}};
use alloc::string::ToString;
use alloc::boxed::Box;

#[no_mangle]
extern "C" fn main() -> ! {
    dsengine::hierarchy::init_script_factory(script_factory);
    dsengine::hierarchy::init_prefab_data(include_bytes!("../../prefab_data.bin"));
    dsengine::main_loop();
}

fn script_factory(id: NonZeroU32) -> Box<dyn Script> {
    match u32::from(id) {
        1 => Box::new(Obj1::default()),
        2 => Box::new(Obj2::default()),
        _ => panic!("Invalid component ID: {}", id)
    }
}

#[derive(Default)]
struct Obj1 {
    cntr: u32
}

impl Script for Obj1 {
    fn start(&mut self, _context: &mut ScriptContext) {
        self.cntr = 4;
    }

    fn update(&mut self, context: &mut ScriptContext) {
        self.cntr += 1;
        console::set_cursor_pos(1, 3);
        //let _d = dsengine::hierarchy::find_by_name("pog");
        //unsafe { dsengine::hierarchy::HIERARCHY.remove(1); }
        //unsafe { dsengine::hierarchy::HIERARCHY.pop(); }
        //ironds::nocash::breakpoint!();
        //ironds::display::console::print(&_d.unwrap().name);
        console::print(&self.cntr.to_string());

        console::set_cursor_pos(1, 10);
        let o2 = context.hierarchy.find_by_script_type::<Obj2>(context.hierarchy.root).unwrap();
        //nocash::print(alloc::format!("{:?}", o2).as_str());
        let o2_bj = context.hierarchy.borrow(o2);
        let o2_scr = o2_bj.cast_script::<Obj2>();
        console::print(&o2_scr.strog);
    }
}

#[derive(Default)]
struct Obj2 {
    cntr: i32,
    strog: String
}

impl dsengine::hierarchy::HasTypeId for Obj2 {
    fn type_id() -> NonZeroU32 {
        NonZeroU32::new(2).unwrap()
    }
}

impl Script for Obj2 {
    fn start(&mut self, _context: &mut ScriptContext) {
        self.cntr = 6;
        self.strog = "flagg;".to_string();
    }

    fn update(&mut self, _context: &mut ScriptContext) {
        self.cntr -= 1;
        console::set_cursor_pos(1, 5);
        console::print(&self.cntr.to_string());
    }
}
