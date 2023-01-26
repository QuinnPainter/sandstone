#![no_std]
#![feature(nonzero_ops)]

extern crate alloc;
use core::num::NonZeroU32;
use hierarchy::Hierarchy;
use ironds as nds;
use alloc::string::String;

pub mod pool;
pub mod hierarchy;

pub fn main_loop() -> ! {
    nds::interrupt::irq_set_handler(Some(inter));
    nds::interrupt::irq_enable(nds::interrupt::IRQFlags::VBLANK);
    nds::display::set_vcount_trigger(100);

    nds::display::console::init_default();
    nds::display::console::print("Hello from Rust on the DS!\n\n");

    let mut hierarchy: Hierarchy = Hierarchy::new();

    let test_obj = hierarchy::run_script_factory(NonZeroU32::new(1).unwrap());
    let _o1handle = hierarchy.add(hierarchy::Node {
        child_handle: None,
        sibling_handle: None,
        name: String::from("Stuff"),
        transform: hierarchy::Transform::default(),
        enabled: false,
        script_data: Some(hierarchy::NodeScriptData {
            type_id: NonZeroU32::new(1).unwrap(),
            script: test_obj  
        })
    }, hierarchy.root);
    let _o2handle = hierarchy.add(hierarchy::Node {
        child_handle: None,
        sibling_handle: None,
        name: String::from("Stuff2"),
        transform: hierarchy::Transform::default(),
        enabled: false,
        script_data: Some(hierarchy::NodeScriptData {
            type_id: NonZeroU32::new(2).unwrap(),
            script: hierarchy::run_script_factory(NonZeroU32::new(2).unwrap())
        })
    }, hierarchy.root);

    hierarchy.run_pending_script_starts();
    hierarchy.temp();

    loop {
        hierarchy.run_script_update();
        hierarchy.run_pending_script_starts();

        nds::interrupt::wait_for_vblank();
    }
}

extern "C" fn inter (f: nds::interrupt::IRQFlags) {
    if f.contains(nds::interrupt::IRQFlags::VBLANK) {
    }
}

pub struct ScriptContext<'a> {
    pub hierarchy: &'a mut Hierarchy
}

pub trait Script: {
    fn update(&mut self, context: &mut ScriptContext);
    fn start(&mut self, context: &mut ScriptContext);
}
