#![no_std]
extern crate alloc;
use ironds as nds;
use alloc::string::String;

pub mod hierarchy;

pub fn main_loop() -> ! {
    nds::interrupt::irq_set_handler(Some(inter));
    nds::interrupt::irq_enable(nds::interrupt::IRQFlags::VBLANK);
    nds::display::set_vcount_trigger(100);

    nds::display::console::init_default();
    nds::display::console::print("Hello from Rust on the DS!\n\n");

    let test_obj = hierarchy::run_component_factory(1);
    hierarchy::HIERARCHY.lock().push(hierarchy::HierarchyItem {
        child_idx: None,
        sibling_idx: None,
        name: String::from("Stuff"),
        transform: hierarchy::Transform::default(),
        enabled: false,
        component: test_obj
    });

    loop {
        for h in hierarchy::HIERARCHY.lock().iter() {
            h.component.clone().borrow_mut().update();
        }
        nds::interrupt::wait_for_vblank();
    }
}

extern "C" fn inter (f: nds::interrupt::IRQFlags) {
    if f.contains(nds::interrupt::IRQFlags::VBLANK) {
    }
}

pub trait Component {
    fn update(&mut self);
    fn start(&mut self);
}
