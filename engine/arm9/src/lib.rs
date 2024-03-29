#![no_std]
#![feature(nonzero_ops)]
#![feature(get_many_mut)]
#![feature(error_in_core)]
#![feature(decl_macro)]

extern crate alloc;
use core::num::NonZeroU32;
use alloc::boxed::Box;
use crate::{hierarchy::Hierarchy, pool::Handle, node::Node};
use ironds as nds;

pub mod pool;
pub mod hierarchy;
pub mod node;
pub mod random;

pub use ironds; // re-export
pub use fixed;
pub use cordic;

/// Type alias for using a Hashbrown HashMap with FxHash
pub type HashMap<K, V> = sandstone_common::HashMap<K, V>;

/// Type alias for using a Hashbrown HashSet with FxHash
pub type HashSet<V> = sandstone_common::HashSet<V>;

pub fn main_loop(game_data_raw: &[u8], script_factory: fn(NonZeroU32) -> Box<dyn Script>) -> ! {
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

    nds::display::console::init_default();
    nds::display::set_sub_display_control(nds::display::DisplayControlSub::new()
        .with_bg_mode(0)
        .with_display_bg0(true)
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

    let mut hierarchy = Hierarchy::new(game_data_raw, script_factory);
    hierarchy.run_extension_init();

    // Load main scene
    hierarchy.set_scene_main();
    hierarchy.process_pending_scene_change();

    loop {
        hierarchy.update_global_positions();
        hierarchy.run_extension_update();
        hierarchy.run_script_update();
        hierarchy.run_pending_script_starts();
        hierarchy.process_pending_destroys();
        hierarchy.process_pending_scene_change();

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

// Could make this an attribute of the camera
#[inline(always)]
pub fn set_bg_colour(colour: u32) {
    unsafe {
        core::ptr::write_volatile(
            ironds::mmio::BG_PALETTE_RAM_BASE_SUB as *mut u16,
            ironds::display::rgb15(colour)
        );
        core::ptr::write_volatile(
            ironds::mmio::BG_PALETTE_RAM_BASE_MAIN as *mut u16,
            ironds::display::rgb15(colour)
        );
    }
}
