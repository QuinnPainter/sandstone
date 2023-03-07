use crate::{pool::Handle, node::Node, hierarchy::Hierarchy};
use ironds::display::{obj, GfxEngine};

pub struct SpriteExtension {
    pub node_handle: Handle<Node>,
    pub graphic_asset: alloc::string::String,
}

pub(crate) fn sprite_init(hierarchy: &Hierarchy) {
    ironds::nocash::print("run");
    for (name, saved_graphic) in hierarchy.saved_prefab_data.graphics.iter() {
        unsafe {
            ironds::nocash::print("loaded");
            core::ptr::copy_nonoverlapping(saved_graphic.tiles.as_ptr(), ironds::mmio::OBJ_RAM_BASE_MAIN as *mut u8, saved_graphic.tiles.len());
            core::ptr::copy_nonoverlapping(saved_graphic.palette.as_ptr(), ironds::mmio::OBJ_PALETTE_RAM_BASE_MAIN as *mut u8, saved_graphic.palette.len());
        }
    }
}

pub(crate) fn sprite_update(hierarchy: &Hierarchy) {
    for (i, sprite) in (0..128).zip(hierarchy.node_ext_pools.sprite_pool.iter().map(|x| Some(x)).chain(core::iter::repeat(None))) {
        if let Some(sprite) = sprite {
            let node = hierarchy.object_pool.borrow(sprite.node_handle);
            obj::set_sprite(GfxEngine::MAIN, i, obj::Sprite::NormalSprite(obj::NormalSprite::new()
                .with_x(node.transform.x as u16)
                .with_y(node.transform.y as u8)
                .with_disable(false)
                .with_h_flip(false)
                .with_v_flip(false)
                .with_mode(0) // Normal mode
                .with_mosaic(false)
                .with_palette_type(false) // 16/16
                .with_shape(0) // square
                .with_size(0) // 8x8
                .with_tile(2)
                .with_priority(0)
                .with_palette(0)
            ));
        } else {
            obj::set_sprite(GfxEngine::MAIN, i, obj::DISABLED_SPRITE);
        }
    }
}

