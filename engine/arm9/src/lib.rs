#![no_std]
extern crate alloc;
use ironds as nds;
use alloc::{string::String, rc::Rc};
use core::cell::RefCell;

mod hierarchy;

static mut COMPONENT_FACTORY: Option<fn(u32) -> Rc<RefCell<dyn Component>>> = None;

pub fn init_component_factory(f: fn(u32) -> Rc<RefCell<dyn Component>>) {
    unsafe { COMPONENT_FACTORY = Some(f); }
}

#[inline]
fn run_component_factory(id: u32) -> Rc<RefCell<dyn Component>> {
    unsafe {
        debug_assert_ne!(COMPONENT_FACTORY, None, "Cannot use Component Factory before initialisation!");
        COMPONENT_FACTORY.unwrap_unchecked()(id)
    }
}

pub fn main_loop() -> ! {
    nds::interrupt::irq_set_handler(Some(inter));
    nds::interrupt::irq_enable(nds::interrupt::IRQFlags::VBLANK);
    nds::display::set_vcount_trigger(100);

    nds::display::console::init_default();
    nds::display::console::print("Hello from Rust on the DS!\n\n");

    let test_obj = run_component_factory(0);
    unsafe {
        hierarchy::HIERARCHY.push(hierarchy::HierarchyItem {
            child_idx: None,
            sibling_idx: None,
            name: String::from("Stuff"),
            transform: hierarchy::Transform::default(),
            enabled: false,
            component: test_obj
        });
    }

    loop {
        unsafe {
            for h in &hierarchy::HIERARCHY {
                h.component.clone().borrow_mut().update();
            }
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
