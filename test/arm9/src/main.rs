#![no_std]
#![no_main]
#![allow(unused_imports)]
extern crate alloc;

use alloc::{string::String, rc::Rc, vec::Vec};
use core::{cell::{RefCell, Ref}, any::Any, num::NonZeroU32};
use dsengine::{hierarchy::{self, Node}, Script, ScriptContext};
use dsengine::pool::Pool;
use ironds::{display::console, sync::{NdsCell, NdsMutex}};
use alloc::string::ToString;
use alloc::boxed::Box;

#[no_mangle]
extern "C" fn main() -> ! {
    dsengine::hierarchy::init_script_factory(script_factory);

    /*let s: RefCell<Obj1> = get_from_id(1, 2);
    let l: Box<dyn Component> = Box::new(Obj1::default());
    l.as_any*/
    dsengine::main_loop();
}

fn script_factory(id: NonZeroU32) -> Box<dyn Script> {
    match u32::from(id) {
        1 => Box::new(Obj1::default()),
        2 => Box::new(Obj2::default()),
        _ => panic!("Invalid component ID: {}", id)
    }
}

/*fn component_factory2(id: u32) -> u32 {
    match id {
        1 => { COMPONENT_VECTORS.lock().obj1.push(Obj1::default()); 1234 },
        _ => panic!("Invalid component ID: {}", id)
    }
}

struct ComponentVectors {
    obj1: Vec<RefCell<Obj1>>,
    obj2: Vec<RefCell<Obj2>>
}

impl ComponentVectors {
    const fn new() -> Self {
        Self {
            obj1: Vec::new(),
            obj2: Vec::new()
        }
    }
}

fn get_from_id<T: Component>(type_id: u32, idx: usize) -> RefCell<T> {
    match id {
        1 => COMPONENT_VECTORS.lock().obj1[idx] as RefCell<T>,
        2 => COMPONENT_VECTORS.lock().obj2[idx] as RefCell<T> ,
        _ => panic!("nope")
    }
}

static COMPONENT_VECTORS: NdsMutex<ComponentVectors> = NdsMutex::new(ComponentVectors::new());
*/
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
        let o2_bj = context.hierarchy.borrow2(o2);
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
