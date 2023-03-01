use crate::{pool::{Pool, Handle}, node::{Node, NodeExtensionPools}};
use ironds::display::{obj, GfxEngine};

pub struct SpriteNode {
    pub node_handle: Handle<Node>,
    pub stuff: u32,
}

pub(crate) fn sprite_update(object_pool: &Pool<Node>, node_ext_pools: &NodeExtensionPools) {
    for (i, sprite) in (0..128).zip(node_ext_pools.sprite_pool.iter().map(|x| Some(x)).chain(core::iter::repeat(None))) {
        if let Some(sprite) = sprite {
            let node = object_pool.borrow(sprite.node_handle);
            obj::set_sprite(GfxEngine::MAIN, i, obj::Sprite::NormalSprite(obj::NormalSprite::new()
                .with_x(node.transform.x as u16)
                .with_y(node.transform.y as u8)
                .with_disable(false)
                .with_h_flip(false)
                .with_v_flip(false)));
        } else {
            obj::set_sprite(GfxEngine::MAIN, i, obj::DISABLED_SPRITE);
        }
    }
}

