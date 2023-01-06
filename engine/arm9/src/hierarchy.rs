use alloc::string::String;
use alloc::rc::Rc;
use alloc::vec::Vec;
use core::num::NonZeroU32;
use core::cell::RefCell;
use crate::Component;
use ironds::sync::NdsMutex;

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

pub static HIERARCHY: NdsMutex<Vec<HierarchyItem>> = NdsMutex::new(Vec::new());

static mut COMPONENT_FACTORY: Option<fn(u32) -> Rc<RefCell<dyn Component>>> = None;

pub fn init_component_factory(f: fn(u32) -> Rc<RefCell<dyn Component>>) {
    unsafe { COMPONENT_FACTORY = Some(f); }
}

#[inline]
#[must_use]
pub (crate) fn run_component_factory(id: u32) -> Rc<RefCell<dyn Component>> {
    unsafe {
        debug_assert_ne!(COMPONENT_FACTORY, None, "Cannot use Component Factory before initialisation!");
        COMPONENT_FACTORY.unwrap_unchecked()(id)
    }
}

#[must_use]
pub fn find_by_name(name: &str) -> Option<usize> {
    HIERARCHY.lock().iter().position(|x| x.name.eq(name))
}
