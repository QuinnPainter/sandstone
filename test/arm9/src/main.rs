#![no_std]
#![no_main]
#![allow(unused_imports)]
extern crate alloc;

use alloc::{string::String, rc::Rc, vec::Vec};
use core::cell::{RefCell, Ref};
use dsengine::hierarchy;
use ironds::{display::console, sync::{NdsCell, NdsMutex}};
use alloc::string::ToString;

#[no_mangle]
extern "C" fn main() -> ! {
    dsengine::hierarchy::init_component_factory(component_factory);

    dsengine::main_loop();
}

fn component_factory(id: u32) -> Rc<RefCell<dyn dsengine::Component>> {
    match id {
        1 => Rc::new(RefCell::new(Obj1::default())),
        2 => Rc::new(RefCell::new(Obj2::default())),
        _ => panic!("Invalid component ID: {}", id)
    }
}

fn component_factory2(id: u32) -> u32 {
    match id {
        1 => { COMPONENT_VECTORS.lock().obj1.push(Obj1::default()); 1234 },
        _ => panic!("Invalid component ID: {}", id)
    }
}

struct ComponentVectors {
    obj1: Vec<Obj1>,
    obj2: Vec<Obj2>
}

impl ComponentVectors {
    const fn new() -> Self {
        Self {
            obj1: Vec::new(),
            obj2: Vec::new()
        }
    }
}

static COMPONENT_VECTORS: NdsMutex<ComponentVectors> = NdsMutex::new(ComponentVectors::new());

#[derive(Default)]
struct Obj1 {
    cntr: u32
}

impl dsengine::Component for Obj1 {
    fn start(&mut self) {
        self.cntr = 4;
    }

    fn update(&mut self) {
        self.cntr += 1;
        console::set_cursor_pos(1, 3);
        //let _d = dsengine::hierarchy::find_by_name("pog");
        //unsafe { dsengine::hierarchy::HIERARCHY.remove(1); }
        //unsafe { dsengine::hierarchy::HIERARCHY.pop(); }
        //ironds::nocash::breakpoint!();
        //ironds::display::console::print(&_d.unwrap().name);
        console::print(&self.cntr.to_string());
    }
}

#[derive(Default)]
struct Obj2 {
    cntr: i32
}

impl dsengine::Component for Obj2 {
    fn start(&mut self) {
        self.cntr = 6;
    }

    fn update(&mut self) {
        self.cntr -= 1;
        console::set_cursor_pos(1, 5);
        console::print(&self.cntr.to_string());
    }
}
