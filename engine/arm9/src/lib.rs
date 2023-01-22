#![no_std]
extern crate alloc;
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

    let test_obj = hierarchy::run_script_factory(1);
    let o1handle = hierarchy.add(hierarchy::HierarchyItem {
        child_idx: None,
        sibling_idx: None,
        name: String::from("Stuff"),
        transform: hierarchy::Transform::default(),
        enabled: false,
        script_type_id: 1,
        script: Some(test_obj)
    });
    let o2handle = hierarchy.add(hierarchy::HierarchyItem {
        child_idx: None,
        sibling_idx: None,
        name: String::from("Stuff2"),
        transform: hierarchy::Transform::default(),
        enabled: false,
        script_type_id: 2,
        script: Some(hierarchy::run_script_factory(2))
    });

    hierarchy.run_pending_script_starts();

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
