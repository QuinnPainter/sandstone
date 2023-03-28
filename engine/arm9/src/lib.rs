#![no_std]
#![feature(nonzero_ops)]
#![feature(get_many_mut)]
#![feature(error_in_core)]
#![feature(decl_macro)]

extern crate alloc;
use crate::{hierarchy::Hierarchy, pool::Handle, node::Node};
use ironds as nds;

pub mod pool;
pub mod hierarchy;
pub mod node;

pub use ironds; // re-export
pub use fixed;

/// Type alias for using a Hashbrown HashMap with FxHash
pub type HashMap<K, V> = sandstone_common::HashMap<K, V>;

/// Type alias for using a Hashbrown HashSet with FxHash
pub type HashSet<V> = sandstone_common::HashSet<V>;

pub fn main_loop() -> ! {
    nds::interrupt::irq_set_handler(Some(inter));
    nds::interrupt::irq_enable(nds::interrupt::IRQFlags::VBLANK);

    // Make sure the 2D graphics engines are turned on
    nds::display::power_on(nds::display::GfxPwr::ALL_2D);

    // set brightness to default level
    nds::display::set_brightness(nds::display::GfxEngine::MAIN, 0);
    nds::display::set_brightness(nds::display::GfxEngine::SUB, 0);

    nds::display::map_vram_block_a(nds::display::vram_type::A::MAIN_OBJ_0);
    nds::display::map_vram_block_d(nds::display::vram_type::D::SUB_OBJ);

    nds::display::set_main_display_control(nds::display::DisplayControlMain::new()
        .with_display_bg0(false)
        .with_display_bg1(false)
        .with_display_bg2(false)
        .with_display_bg3(false)
        .with_display_obj(true)
        .with_bm_obj_1d_bound(0)
        .with_obj_during_hblank(false)
        .with_bg_ext_pal_enabled(false)
        .with_obj_ext_pal_enabled(false)
        .with_forced_blank(false)
        .with_tile_obj_mapping(true) // 1D mapping
        .with_display_mode(1) // normal BG / OBJ display
    );

    nds::display::set_sub_display_control(nds::display::DisplayControlSub::new()
        .with_display_bg0(false)
        .with_display_bg1(false)
        .with_display_bg2(false)
        .with_display_bg3(false)
        .with_display_obj(true)
        .with_obj_during_hblank(false)
        .with_bg_ext_pal_enabled(false)
        .with_obj_ext_pal_enabled(false)
        .with_forced_blank(false)
        .with_tile_obj_mapping(true) // 1D mapping
        .with_display_mode(1) // normal BG / OBJ display
    );

    //nds::display::console::init_default();
    //nds::display::console::print("Hello from Rust on the DS!\n\n");

    let mut hierarchy: Hierarchy = Hierarchy::new();
    hierarchy.run_extension_init();

    // Load main scene
    hierarchy.spawn_prefab(&hierarchy.saved_prefab_data.main_graph.clone(), hierarchy.root);
    hierarchy.run_pending_script_starts();
    //hierarchy.pretty_print_hierarchy_structure();

    loop {
        hierarchy.update_global_positions();
        hierarchy.run_extension_update();
        hierarchy.run_script_update();
        hierarchy.run_pending_script_starts();
        hierarchy.process_pending_destroys();

        nds::interrupt::wait_for_vblank();
    }
}

extern "C" fn inter (f: nds::interrupt::IRQFlags) {
    if f.contains(nds::interrupt::IRQFlags::VBLANK) {
    }
}

pub struct ScriptContext<'a> {
    pub hierarchy: &'a mut Hierarchy,
    pub handle: Handle<Node>,
}

pub trait Script: {
    fn update(&mut self, context: &mut ScriptContext);
    fn start(&mut self, context: &mut ScriptContext);
}

pub macro register_script ($script:ident, $num:literal) {
    #[doc = concat!("{script_type_id=", $num, "}")]
    impl sandstone::hierarchy::HasTypeId for $script {
        fn type_id() -> core::num::NonZeroU32 {
            core::num::NonZeroU32::new($num).unwrap()
        }
    }
}
