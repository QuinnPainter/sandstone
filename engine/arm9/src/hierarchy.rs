use alloc::string::String;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::num::NonZeroU32;
use core::cell::RefCell;
use crate::Component;

#[derive(Eq, PartialEq, Clone, Copy, Default, Debug)]
pub struct Transform {
    pub x: u32,
    pub y: u32,
}

pub struct HierarchyItem {
    pub child_idx: Option<NonZeroU32>,
    pub sibling_idx: Option<NonZeroU32>,
    pub name: String,
    pub transform: Transform,
    pub component: Rc<RefCell<dyn Component>>,
    pub enabled: bool,
}

pub static mut HIERARCHY: Vec<HierarchyItem> = Vec::new();
