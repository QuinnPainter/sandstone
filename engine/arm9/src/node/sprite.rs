use crate::{pool::Handle, node::{Node, camera::{ActiveCameras, CameraExtension}}, hierarchy::Hierarchy, HashMap};
use alloc::string::String;
use ironds::display::{obj, GfxEngine};
use sandstone_common::{SavedPrefabs, SpriteSize};

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

fn sprite_size_to_shape_and_size(ss: SpriteSize) -> (u8, u8) {
    match ss {
        SpriteSize::_8x8 => (0, 0),
        SpriteSize::_16x16 => (0, 1),
        SpriteSize::_32x32 => (0, 2),
        SpriteSize::_64x64 => (0, 3),
        SpriteSize::_16x8 => (1, 0),
        SpriteSize::_32x8 => (1, 1),
        SpriteSize::_32x16 => (1, 2),
        SpriteSize::_64x32 => (1, 3),
        SpriteSize::_8x16 => (2, 0),
        SpriteSize::_8x32 => (2, 1),
        SpriteSize::_16x32 => (2, 2),
        SpriteSize::_32x64 => (2, 3),
    }
}

impl SpriteExtensionHandler {
    pub fn new() -> Self {
        Self {
            sprite_vram_map: HashMap::default(),
        }
    }

    pub fn sprite_init(&mut self, saved_prefab_data: &SavedPrefabs) {
        self.sprite_init_for_engine(saved_prefab_data, GfxEngine::MAIN);
        self.sprite_init_for_engine(saved_prefab_data, GfxEngine::SUB);
    }

    // this really should be happening in Vblank handler, given that OAM is only accessible during vblank
    pub fn sprite_update(&self, hierarchy: &Hierarchy, cameras: ActiveCameras) {
        if let Some(camera) = cameras.main {
            self.sprite_update_for_engine(hierarchy, GfxEngine::MAIN, camera);
        }
        if let Some(camera) = cameras.sub {
            self.sprite_update_for_engine(hierarchy, GfxEngine::SUB, camera);
        }
    }

    fn sprite_init_for_engine(&mut self, saved_prefab_data: &SavedPrefabs, engine: GfxEngine) {
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

        let (tile_ram_base, pal_ram_base) = match engine {
            GfxEngine::MAIN => (ironds::mmio::OBJ_RAM_BASE_MAIN as *mut u8, ironds::mmio::OBJ_PALETTE_RAM_BASE_MAIN as *mut u8),
            GfxEngine::SUB => (ironds::mmio::OBJ_RAM_BASE_SUB as *mut u8, ironds::mmio::OBJ_PALETTE_RAM_BASE_SUB as *mut u8),
        };
        let mut cur_tile_ram_ptr = tile_ram_base;
        let mut cur_pal_ram_ptr = pal_ram_base;
        for (name, saved_graphic) in saved_prefab_data.graphics.iter() {
            self.sprite_vram_map.insert(name.clone(), SpriteVramMapping {
                tile_index: ((cur_tile_ram_ptr as usize - tile_ram_base as usize) / SIZEOF_TILE) as u16,
                pal_index: ((cur_pal_ram_ptr as usize - pal_ram_base as usize) / SIZEOF_PALETTE) as u8,
            });
            unsafe {
                // todo: check for overflow of tile / palette ram
                let tile_end = cur_tile_ram_ptr.add(saved_graphic.tiles.len());
                let pal_end = cur_pal_ram_ptr.add(saved_graphic.palette.len());
                core::ptr::copy_nonoverlapping(saved_graphic.tiles.as_ptr(), cur_tile_ram_ptr, saved_graphic.tiles.len());
                core::ptr::copy_nonoverlapping(saved_graphic.palette.as_ptr(), cur_pal_ram_ptr, saved_graphic.palette.len());
                cur_tile_ram_ptr = align_to(tile_end, SIZEOF_TILE);
                cur_pal_ram_ptr = align_to(pal_end, SIZEOF_PALETTE);
            }
        }
    }

    fn sprite_update_for_engine(&self, hierarchy: &Hierarchy, engine: GfxEngine, camera: Handle<CameraExtension>) {
        let camera = hierarchy.node_ext_pools.camera_pool.borrow(camera);
        let camera_node = hierarchy.object_pool.borrow(camera.node_handle);
        let (cam_x, cam_y) = (camera_node.global_transform.x, camera_node.global_transform.y);

        let mut cur_sprite_index = 0;
        for sprite in hierarchy.node_ext_pools.sprite_pool.iter() {
            let node = hierarchy.object_pool.borrow(sprite.node_handle);
            if node.global_enabled == false { continue; }

            let vram_mapping = self.sprite_vram_map[&sprite.graphic_asset];
            let (shape, size) = sprite_size_to_shape_and_size(hierarchy.saved_prefab_data.graphics[&sprite.graphic_asset].size);

            let screen_x = node.global_transform.x - cam_x;
            let screen_y = node.global_transform.y - cam_y;
            if screen_y < 192 && screen_y > -64 && screen_x < 256 && screen_x > -128 {
                let screen_x = (screen_x.to_num::<i32>() & 0x1FF) as u16;
                let screen_y = (screen_y.to_num::<i32>() & 0xFF) as u8;

                obj::set_sprite(engine, cur_sprite_index, obj::Sprite::NormalSprite(obj::NormalSprite::new()
                    .with_x(screen_x)
                    .with_y(screen_y)
                    .with_disable(false)
                    .with_h_flip(false)
                    .with_v_flip(false)
                    .with_mode(0) // Normal mode
                    .with_mosaic(false)
                    .with_palette_type(false) // 16/16
                    .with_shape(shape) // square
                    .with_size(size) // 8x8
                    .with_tile(vram_mapping.tile_index)
                    .with_priority(0)
                    .with_palette(vram_mapping.pal_index)
                ));
                cur_sprite_index += 1;
            }
        }
        for i in cur_sprite_index..128 {
            obj::set_sprite(engine, i, obj::DISABLED_SPRITE);
        }
    }
}

