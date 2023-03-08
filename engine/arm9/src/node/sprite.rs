use crate::{pool::Handle, node::Node, hierarchy::Hierarchy, HashMap};
use alloc::string::String;
use ironds::display::{obj, GfxEngine};
use sandstone_common::SavedPrefabs;

// Assumes 16 palette / 16 colour mode.
const SIZEOF_PALETTE: usize = 2 * 16;
const SIZEOF_TILE: usize = (8 * 8) / 2;

pub struct SpriteExtension {
    pub node_handle: Handle<Node>,
    pub graphic_asset: String,
}

pub(crate) struct SpriteExtensionHandler {
    sprite_vram_map: HashMap<String, SpriteVramMapping>,
}

#[derive(Copy, Clone)]
struct SpriteVramMapping {
    tile_index: u16,
    pal_index: u8,
}

impl SpriteExtensionHandler {
    pub fn new() -> Self {
        Self {
            sprite_vram_map: HashMap::default(),
        }
    }

    pub fn sprite_init(&mut self, saved_prefab_data: &SavedPrefabs) {
        #[inline(always)]
        fn align_to(ptr: *mut u8, align: usize) -> *mut u8 {
            let align_mask = align - 1;
            let ptr_u = ptr as usize;
            if ptr_u & align_mask != 0 {
                (ptr_u + (align - (ptr_u & align_mask))) as *mut u8
            } else {
                ptr
            }
        }

        let tile_ram_base = ironds::mmio::OBJ_RAM_BASE_MAIN as *mut u8;
        let pal_ram_base = ironds::mmio::OBJ_PALETTE_RAM_BASE_MAIN as *mut u8;
        let mut cur_tile_ram_ptr = tile_ram_base;
        let mut cur_pal_ram_ptr = pal_ram_base;
        for (name, saved_graphic) in saved_prefab_data.graphics.iter() {
            unsafe {
                // todo: check for overflow of tile / palette ram
                let tile_end = cur_tile_ram_ptr.add(saved_graphic.tiles.len());
                let pal_end = cur_pal_ram_ptr.add(saved_graphic.palette.len());
                core::ptr::copy_nonoverlapping(saved_graphic.tiles.as_ptr(), cur_tile_ram_ptr, saved_graphic.tiles.len());
                core::ptr::copy_nonoverlapping(saved_graphic.palette.as_ptr(), cur_pal_ram_ptr, saved_graphic.palette.len());
                cur_tile_ram_ptr = align_to(tile_end, SIZEOF_TILE);
                cur_pal_ram_ptr = align_to(pal_end, SIZEOF_PALETTE);
            }
            self.sprite_vram_map.insert(name.clone(), SpriteVramMapping {
                tile_index: ((cur_tile_ram_ptr as usize - tile_ram_base as usize) / SIZEOF_TILE) as u16,
                pal_index: ((cur_pal_ram_ptr as usize - pal_ram_base as usize) / SIZEOF_PALETTE) as u8,
            });
        }
    }

    pub fn sprite_update(&self, hierarchy: &Hierarchy) {
        for (i, sprite) in (0..128).zip(hierarchy.node_ext_pools.sprite_pool.iter().map(|x| Some(x)).chain(core::iter::repeat(None))) {
            if let Some(sprite) = sprite {
                let node = hierarchy.object_pool.borrow(sprite.node_handle);
                let vram_mapping = self.sprite_vram_map[&sprite.graphic_asset];
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
                    .with_tile(vram_mapping.tile_index)
                    .with_priority(0)
                    .with_palette(vram_mapping.pal_index)
                ));
            } else {
                obj::set_sprite(GfxEngine::MAIN, i, obj::DISABLED_SPRITE);
            }
        }
    }
}

