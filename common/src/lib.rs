#![no_std]
extern crate alloc;
use core::num::NonZeroU32;
use alloc::{string::String, vec::Vec};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SavedSpriteExtension {
}

#[derive(Serialize, Deserialize)]
pub enum SavedNodeExtension {
    None,
    Sprite(SavedSpriteExtension),
}

#[derive(Serialize, Deserialize)]
pub struct SavedTransform {
    pub x: u32,
    pub y: u32
}

#[derive(Serialize, Deserialize)]
pub struct SavedNode {
    pub child_index: Option<NonZeroU32>,
    pub parent_index: Option<u32>, // not technically necessary to save, but makes things easier when deserialising
    pub sibling_index: Option<NonZeroU32>,
    pub name: String,
    pub transform: SavedTransform,
    pub node_extension: SavedNodeExtension,
    pub script_type_id: Option<NonZeroU32>,
    pub enabled: bool
}

#[derive(Serialize, Deserialize)]
pub struct SavedNodeGraph {
    pub nodes: Vec<SavedNode>
}

#[derive(Serialize, Deserialize)]
pub struct SavedPrefabs (pub Vec<SavedNodeGraph>);

pub fn serialize_prefabs(h: &SavedPrefabs) -> Vec<u8> {
    postcard::to_allocvec(h).unwrap()
}

pub fn deserialize_prefabs(h: &[u8]) -> SavedPrefabs {
    postcard::from_bytes(h).unwrap()
}
