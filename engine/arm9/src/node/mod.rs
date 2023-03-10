use core::num::NonZeroU32;
use alloc::{string::String, boxed::Box};
use crate::{Script, pool::{Pool, Handle}, hierarchy::HasTypeId};

pub mod sprite;
pub mod camera;
pub mod rect_collider;

#[derive(Eq, PartialEq, Clone, Copy, Default, Debug)]
pub struct Transform {
    pub x: fixed::types::I20F12,
    pub y: fixed::types::I20F12,
}

#[derive(Clone, Copy, Debug)]
pub enum NodeExtensionHandle {
    None,
    Sprite(Handle<sprite::SpriteExtension>),
    Camera(Handle<camera::CameraExtension>),
    RectCollider(Handle<rect_collider::RectColliderExtension>),
}

impl NodeExtensionHandle {
    pub(crate) fn from_saved(
        pools: &mut NodeExtensionPools,
        node_handle: Handle<Node>,
        saved_extension: &sandstone_common::SavedNodeExtension) -> Self
    {
        match saved_extension {
            sandstone_common::SavedNodeExtension::None => NodeExtensionHandle::None,
            sandstone_common::SavedNodeExtension::Sprite(s) => {
                NodeExtensionHandle::Sprite(pools.sprite_pool.add(sprite::SpriteExtension {
                    graphic_asset: s.graphic_asset.clone(),
                    node_handle,
                }))
            },
            sandstone_common::SavedNodeExtension::Camera(c) => {
                NodeExtensionHandle::Camera(pools.camera_pool.add(camera::CameraExtension {
                    node_handle,
                    active_main: c.active_main,
                    active_sub: c.active_sub,
                }))
            },
            sandstone_common::SavedNodeExtension::RectCollider(c) => {
                NodeExtensionHandle::RectCollider(pools.rect_collider_pool.add(rect_collider::RectColliderExtension {
                    node_handle,
                    width: c.width,
                    height: c.height,
                }))
            },
        }
    }
}

pub(crate) struct NodeExtensionPools {
    pub sprite_pool: Pool<sprite::SpriteExtension>,
    pub camera_pool: Pool<camera::CameraExtension>,
    pub rect_collider_pool: Pool<rect_collider::RectColliderExtension>,
}

impl NodeExtensionPools {
    pub const fn new() -> Self {
        Self {
            sprite_pool: Pool::new(),
            camera_pool: Pool::new(),
            rect_collider_pool: Pool::new(),
        }
    }
}

pub struct NodeScriptData {
    pub type_id: NonZeroU32,
    pub script: Box<dyn Script>,
}

pub struct Node {
    pub child_handle: Option<Handle<Node>>, // todo: maybe could be more efficient, could just be Index without Generation
    pub parent_handle: Option<Handle<Node>>, // should only be None on root node
    pub sibling_handle: Option<Handle<Node>>,
    pub name: String,
    pub transform: Transform,
    pub node_extension: NodeExtensionHandle,
    pub script_data: Option<NodeScriptData>,
    pub enabled: bool,
}

impl Node {
    pub fn cast_script<T>(&self) -> &T
    where T: Script + HasTypeId {
        let s_data = self.script_data.as_ref().expect("Tried to cast_script on an object which has no Script");
        assert_eq!(<T as HasTypeId>::type_id(), s_data.type_id, "Tried to cast_script with mismatching types");
        unsafe { &*(s_data.script.as_ref() as *const dyn Script as *const T) }
    }

    pub fn cast_script_mut<T>(&mut self) -> &mut T
    where T: Script + HasTypeId {
        let s_data = self.script_data.as_mut().expect("Tried to cast_script on an object which has no Script");
        assert_eq!(<T as HasTypeId>::type_id(), s_data.type_id, "Tried to cast_script with mismatching types");
        unsafe { &mut *(s_data.script.as_mut() as *mut dyn Script as *mut T) }
    }
}
