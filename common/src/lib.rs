#![no_std]
extern crate alloc;
use alloc::{string::String, vec::Vec};
use core::num::NonZeroU32;
use serde::{Deserialize, Serialize};

pub type HashMap<K, V> = hashbrown::HashMap<K, V, core::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
pub type HashSet<V> = hashbrown::HashSet<V, core::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

#[derive(Serialize, Deserialize)]
pub struct SavedSpriteExtension {
    pub graphic_asset: String,
}

#[derive(Serialize, Deserialize)]
pub enum SavedNodeExtension {
    None,
    Sprite(SavedSpriteExtension),
}

#[derive(Serialize, Deserialize)]
pub struct SavedTransform {
    pub x: u32,
    pub y: u32,
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
    pub enabled: bool,
}

#[derive(Serialize, Deserialize)]
pub struct SavedNodeGraph {
    pub nodes: Vec<SavedNode>,
}

#[derive(Serialize, Deserialize)]
pub struct SavedGraphic {
    pub tiles: Vec<u8>,
    pub palette: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub struct SavedPrefabs {
    pub graphs: Vec<SavedNodeGraph>,
    pub graphics: HashMap<String, SavedGraphic>,
}

pub fn serialize<T>(h: &T) -> Vec<u8>
where
    T: Serialize,
{
    postcard::to_allocvec(h).unwrap()
}

pub fn deserialize<'a, T>(h: &'a [u8]) -> T
where
    T: Deserialize<'a>,
{
    postcard::from_bytes(h).unwrap()
}
