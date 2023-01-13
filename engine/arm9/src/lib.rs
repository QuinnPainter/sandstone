#![no_std]
extern crate alloc;
use hierarchy::HierarchyItem;
use ironds as nds;
use alloc::string::String;
use pool::Pool;

pub mod pool;
pub mod hierarchy;

pub fn main_loop() -> ! {
    nds::interrupt::irq_set_handler(Some(inter));
    nds::interrupt::irq_enable(nds::interrupt::IRQFlags::VBLANK);
    nds::display::set_vcount_trigger(100);

    nds::display::console::init_default();
    nds::display::console::print("Hello from Rust on the DS!\n\n");

    let mut hierarchy: Pool<HierarchyItem> = Pool::new();

    let test_obj = hierarchy::run_component_factory(1);
    let o1handle = hierarchy.add(hierarchy::HierarchyItem {
        child_idx: None,
        sibling_idx: None,
        name: String::from("Stuff"),
        transform: hierarchy::Transform::default(),
        enabled: false,
        script_type_id: 1,
        script: test_obj
    });
    let o2handle = hierarchy.add(hierarchy::HierarchyItem {
        child_idx: None,
        sibling_idx: None,
        name: String::from("Stuff2"),
        transform: hierarchy::Transform::default(),
        enabled: false,
        script_type_id: 2,
        script: hierarchy::run_component_factory(2)
    });

    for i in 0..hierarchy.vec_len() {
        if let Some((item_ticket, mut item)) = hierarchy.try_take_by_index(i) {
            let mut context = ScriptContext { hierarchy: &mut hierarchy };
            item.script.start(&mut context);
            hierarchy.put_back(item_ticket, item);
        }
    }

    loop {
        for i in 0..hierarchy.vec_len() {
            if let Some((item_ticket, mut item)) = hierarchy.try_take_by_index(i) {
                let mut context = ScriptContext { hierarchy: &mut hierarchy };
                item.script.update(&mut context);
                hierarchy.put_back(item_ticket, item);
            }
        }
        nds::interrupt::wait_for_vblank();
    }
}

extern "C" fn inter (f: nds::interrupt::IRQFlags) {
    if f.contains(nds::interrupt::IRQFlags::VBLANK) {
    }
}

pub struct ScriptContext<'a> {
    pub hierarchy: &'a mut Pool<HierarchyItem>
}

pub trait Script: {
    fn update(&mut self, context: &mut ScriptContext);
    fn start(&mut self, context: &mut ScriptContext);
}
