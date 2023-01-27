#![no_std]
#![feature(nonzero_ops)]

extern crate alloc;
use hierarchy::Hierarchy;
use ironds as nds;

pub mod pool;
pub mod hierarchy;

pub fn main_loop() -> ! {
    nds::interrupt::irq_set_handler(Some(inter));
    nds::interrupt::irq_enable(nds::interrupt::IRQFlags::VBLANK);
    nds::display::set_vcount_trigger(100);

    nds::display::console::init_default();
    nds::display::console::print("Hello from Rust on the DS!\n\n");

    let mut hierarchy: Hierarchy = Hierarchy::new();

    hierarchy.spawn_prefab(0, hierarchy.root);
    hierarchy.spawn_prefab(1, hierarchy.root);
    hierarchy.run_pending_script_starts();
    //hierarchy.pretty_print_hierarchy_structure();

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
