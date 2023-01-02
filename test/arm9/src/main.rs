#![no_std]
#![no_main]
extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use dsengine;
use ironds::display::console;
use alloc::string::ToString;

#[no_mangle]
extern "C" fn main() -> ! {
    dsengine::init_component_factory(component_factory);

    dsengine::main_loop();
}

fn component_factory(id: u32) -> Rc<RefCell<dyn dsengine::Component>> {
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
        console::print(&self.cntr.to_string());
    }
}
