#![no_std]
#![no_main]
extern crate alloc;

use alloc::{string::String, rc::Rc};
use core::cell::RefCell;
use dsengine::hierarchy;
use ironds::display::console;
use alloc::string::ToString;

#[no_mangle]
extern "C" fn main() -> ! {
    dsengine::hierarchy::init_component_factory(component_factory);

    hierarchy::HIERARCHY.lock().push(hierarchy::HierarchyItem {
        child_idx: None,
        sibling_idx: None,
        name: String::from("pog"),
        transform: hierarchy::Transform::default(),
        enabled: false,
        component: component_factory(0)
    });

    dsengine::main_loop();
}

fn component_factory(id: u32) -> Rc<RefCell<dyn dsengine::Component>> {
    if id == 0 {}
    return Rc::new(RefCell::new(Obj1::default()));
}

#[derive(Default)]
pub struct Obj1 {
    pub cntr: u32
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
