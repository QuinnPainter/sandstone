use alloc::{string::String, boxed::Box, vec::Vec};
use core::num::NonZeroU32;
use crate::{Script, pool::{Pool, Handle}, ScriptContext};

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

pub struct Hierarchy {
    object_pool: Pool<HierarchyItem>,
    to_start_stack: Vec<Handle<HierarchyItem>>
}

impl Hierarchy {
    pub const fn new() -> Self {
        Self { object_pool: Pool::new(), to_start_stack: Vec::new() }
    }

    pub fn add(&mut self, item: HierarchyItem) {
        let handle = self.object_pool.add(item);
        self.to_start_stack.push(handle);
    }

    #[must_use]
    pub fn find_by_script_type<T>(&mut self) -> Option<&mut T>
    where T: Script + HasTypeId {
        let s = self.object_pool.iter_mut().find(|x| x.script_type_id == <T as HasTypeId>::type_id());
        let s_2 = s.unwrap();
        let s_3 = s_2.script.as_mut();
        unsafe { Some( &mut *(s_3 as *mut dyn Script as *mut T) ) }
    }

    /*#[must_use]
    pub fn find_by_name(&mut self, search_root: Handle<Hierarchy>, name: &str) -> Option<Handle<HierarchyItem>> {
        // todo: fast path for situation where search root is graph root node
        // could iterate over vec sequentially instead of following tree
        let s = self.object_pool.iter_mut().find(|x| x.name == name);
        let s_2 = s.unwrap();
    }*/

    pub(crate) fn run_pending_script_starts(&mut self) {
        while self.run_one_pending_script_start() {}
    }

    pub(crate) fn run_one_pending_script_start(&mut self) -> bool{
        if let Some(handle) = self.to_start_stack.pop() {
            // this could return None if an object was immediately destroyed after creating it
            if let Some((item_ticket, mut item)) = self.object_pool.try_take(handle) {
                let mut context = ScriptContext { hierarchy: self };
                item.script.start(&mut context);
                self.object_pool.put_back(item_ticket, item);
            }
            return true;
        }
        false
    }

    pub(crate) fn run_script_update(&mut self) {
        for i in 0..self.object_pool.vec_len() {
            if let Some((item_ticket, mut item)) = self.object_pool.try_take_by_index(i) {
                let mut context = ScriptContext { hierarchy: self };
                item.script.update(&mut context);
                self.object_pool.put_back(item_ticket, item);
            }
        }
    }
}

static mut SCRIPT_FACTORY: Option<fn(u32) -> Box<dyn Script>> = None;

pub fn init_script_factory(f: fn(u32) -> Box<dyn Script>) {
    unsafe { SCRIPT_FACTORY = Some(f); }
}

#[inline]
#[must_use]
pub (crate) fn run_script_factory(id: u32) -> Box<dyn Script> {
    unsafe {
        debug_assert_ne!(SCRIPT_FACTORY, None, "Cannot use Script Factory before initialisation!");
        SCRIPT_FACTORY.unwrap_unchecked()(id)
    }
}

pub trait HasTypeId {
    fn type_id() -> u32;
}
