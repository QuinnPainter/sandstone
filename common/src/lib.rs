#![no_std]
extern crate alloc;
use core::num::NonZeroU32;
use alloc::{string::String, vec::Vec};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct SavedTransform {
    pub x: u32,
    pub y: u32
}

#[derive(Serialize, Deserialize)]
pub struct SavedNode {
    pub child_index: Option<NonZeroU32>,
    pub sibling_index: Option<NonZeroU32>,
    pub name: String,
    pub transform: SavedTransform,
    pub script_type_id: Option<NonZeroU32>,
    pub enabled: bool
}

#[derive(Serialize, Deserialize)]
pub struct SavedNodeGraph {
    pub nodes: Vec<SavedNode>
}

pub fn do_serialize(h: &SavedNodeGraph) -> Vec<u8> {
    postcard::to_allocvec(h).unwrap()
}

pub fn do_deserialize(h: &[u8]) -> SavedNodeGraph {
    postcard::from_bytes(h).unwrap()
}
