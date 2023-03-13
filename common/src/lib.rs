#![no_std]
extern crate alloc;
use alloc::{string::String, vec::Vec};
use core::num::NonZeroU32;
use serde::{Deserialize, Serialize};

pub type HashMap<K, V> = hashbrown::HashMap<K, V, core::hash::BuildHasherDefault<rustc_hash::FxHasher>>;
pub type HashSet<V> = hashbrown::HashSet<V, core::hash::BuildHasherDefault<rustc_hash::FxHasher>>;

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpriteSize {
    #[default]
    _8x8, // Square
    _16x16,
    _32x32,
    _64x64,
    _16x8, // Horizontal
    _32x8,
    _32x16,
    _64x32,
    _8x16, // Vertical
    _8x32,
    _16x32,
    _32x64,
}

impl core::fmt::Display for SpriteSize {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            SpriteSize::_8x8 => write!(f, "8x8"),
            SpriteSize::_16x16 => write!(f, "16x16"),
            SpriteSize::_32x32 => write!(f, "32x32"),
            SpriteSize::_64x64 => write!(f, "64x64"),
            SpriteSize::_16x8 => write!(f, "16x8"),
            SpriteSize::_32x8 => write!(f, "32x8"),
            SpriteSize::_32x16 => write!(f, "32x16"),
            SpriteSize::_64x32 => write!(f, "64x32"),
            SpriteSize::_8x16 => write!(f, "8x16"),
            SpriteSize::_8x32 => write!(f, "8x32"),
            SpriteSize::_16x32 => write!(f, "16x32"),
            SpriteSize::_32x64 => write!(f, "32x64"),
        }
    }
}

impl SpriteSize {
    pub fn to_dimensions(self) -> (u8, u8) {
        match self {
            SpriteSize::_8x8 => (8, 8),
            SpriteSize::_16x16 => (16, 16),
            SpriteSize::_32x32 => (32, 32),
            SpriteSize::_64x64 => (64, 64),
            SpriteSize::_16x8 => (16, 8),
            SpriteSize::_32x8 => (32, 8),
            SpriteSize::_32x16 => (32, 16),
            SpriteSize::_64x32 => (64, 32),
            SpriteSize::_8x16 => (8, 16),
            SpriteSize::_8x32 => (8, 32),
            SpriteSize::_16x32 => (16, 32),
            SpriteSize::_32x64 => (32, 64),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedSpriteExtension {
    pub graphic_asset: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedCameraExtension {
    pub active_main: bool,
    pub active_sub: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SavedNodeExtension {
    None,
    Sprite(SavedSpriteExtension),
    Camera(SavedCameraExtension),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedTransform {
    pub x: fixed::types::I20F12,
    pub y: fixed::types::I20F12,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedNodeGraph {
    pub nodes: Vec<SavedNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedGraphic {
    pub tiles: Vec<u8>,
    pub palette: Vec<u8>,
    pub size: SpriteSize,
}

#[derive(Debug, Serialize, Deserialize)]
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
