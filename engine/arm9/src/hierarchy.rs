use alloc::{string::String, boxed::Box};
use core::num::NonZeroU32;
use crate::Script;

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
    pub script_type_id: u32,
    pub script: Box<dyn Script>,
    pub enabled: bool,
}

static mut COMPONENT_FACTORY: Option<fn(u32) -> Box<dyn Script>> = None;

pub fn init_component_factory(f: fn(u32) -> Box<dyn Script>) {
    unsafe { COMPONENT_FACTORY = Some(f); }
}

#[inline]
#[must_use]
pub (crate) fn run_component_factory(id: u32) -> Box<dyn Script> {
    unsafe {
        debug_assert_ne!(COMPONENT_FACTORY, None, "Cannot use Component Factory before initialisation!");
        COMPONENT_FACTORY.unwrap_unchecked()(id)
    }
}

/*#[must_use]
pub fn find_by_name(name: &str) -> Option<usize> {
    HIERARCHY.lock().iter().position(|x| x.name.eq(name))
}*/

pub trait HasTypeId {
    fn type_id() -> u32;
}

#[must_use]
pub fn find_by_component_type<T>(hierarchy: &mut crate::pool::Pool<HierarchyItem>) -> Option<&mut T>
where T: Script + HasTypeId {
    let s = hierarchy.iter_mut().find(|x| x.script_type_id == <T as HasTypeId>::type_id());
    let s_2 = s.unwrap();
    let s_3 = s_2.script.as_mut();
    unsafe { Some( &mut *(s_3 as *mut dyn Script as *mut T) ) }
}
