use alloc::{string::String, boxed::Box, vec::Vec};
use crate::{Script, pool::{Pool, Handle}, ScriptContext};

#[derive(Eq, PartialEq, Clone, Copy, Default, Debug)]
pub struct Transform {
    pub x: u32,
    pub y: u32,
}

pub struct HierarchyItem {
    pub child_handle: Option<Handle<HierarchyItem>>, // todo: maybe could be more efficient, could just be Index without Generation
    pub sibling_handle: Option<Handle<HierarchyItem>>,
    pub name: String,
    pub transform: Transform,
    pub script_type_id: u32, // only valid if script is Some
    pub script: Option<Box<dyn Script>>,
    pub enabled: bool,
}

impl HierarchyItem {
    pub fn cast_script<T>(&self) -> &T
    where T: Script + HasTypeId {
        if <T as HasTypeId>::type_id() != self.script_type_id {
            panic!("nivalid type");
        }
        if self.script.is_none() {
            panic!("no script")
        }
        unsafe { &*(self.script.as_ref().unwrap_unchecked().as_ref() as *const dyn Script as *const T) }
    }

    pub fn cast_script_mut<T>(&mut self) -> &mut T
    where T: Script + HasTypeId {
        if <T as HasTypeId>::type_id() != self.script_type_id {
            panic!("nivalid type");
        }
        if self.script.is_none() {
            panic!("no script")
        }
        unsafe { &mut *(self.script.as_mut().unwrap_unchecked().as_mut() as *mut dyn Script as *mut T) }
    }
}

pub struct Hierarchy {
    pub root: Handle<HierarchyItem>,
    object_pool: Pool<HierarchyItem>,
    to_start_stack: Vec<Handle<HierarchyItem>>
}

impl Hierarchy {
    pub fn new() -> Self {
        let mut object_pool: Pool<HierarchyItem> = Pool::new();
        let root = object_pool.add(HierarchyItem {
            child_handle: None,
            sibling_handle: None,
            name: String::new(),
            transform: Transform::default(),
            script_type_id: 0,
            script: None,
            enabled: true
        });

        Self {
            root,
            object_pool,
            to_start_stack: Vec::new(),
        }
    }

    pub fn add(&mut self, item: HierarchyItem, parent: Handle<HierarchyItem>) {
        let handle = self.object_pool.add(item);
        let parent_obj = self.object_pool.borrow_mut(parent);
        self.object_pool.borrow_mut(handle).sibling_handle = parent_obj.child_handle.replace(handle);

        self.to_start_stack.push(handle);
    }

    #[inline]
    #[must_use]
    pub fn borrow2(&self, handle: Handle<HierarchyItem>) -> &HierarchyItem {
        self.object_pool.borrow(handle)
    }

    // todo: recursive search?
    // could have fast path for situation where search root is graph root node
    // as we can iterate over vec sequentially instead of following the tree

    #[must_use]
    pub fn find_by_name(&mut self, search_root: Handle<HierarchyItem>, name: &str) -> Option<Handle<HierarchyItem>> {
        self.find(search_root, |x| x.name == name)
    }

    #[must_use]
    pub fn find_by_script_type<T>(&mut self, search_root: Handle<HierarchyItem>) -> Option<Handle<HierarchyItem>>
    where T: Script + HasTypeId {
        self.find(search_root, |x| x.script_type_id == <T as HasTypeId>::type_id())
    }

    #[must_use]
    pub fn find<P>(&mut self, search_root: Handle<HierarchyItem>, mut predicate: P) -> Option<Handle<HierarchyItem>>
    where P: FnMut(&HierarchyItem) -> bool, {
        let mut cur_node_handle = self.object_pool.borrow(search_root).child_handle?;
        loop {
            let cur_node = self.object_pool.borrow(cur_node_handle);
            if predicate(cur_node) {
                return Some(cur_node_handle);
            }
            cur_node_handle = cur_node.sibling_handle?;
        }
    }

    pub(crate) fn run_pending_script_starts(&mut self) {
        while self.run_one_pending_script_start() {}
    }

    pub(crate) fn run_one_pending_script_start(&mut self) -> bool{
        if let Some(handle) = self.to_start_stack.pop() {
            // this could return None if an object was immediately destroyed after creating it
            if let Some((item_ticket, mut item)) = self.object_pool.try_take(handle) {
                if let Some(script) = &mut item.script {
                    let mut context = ScriptContext { hierarchy: self };
                    script.start(&mut context);
                }
                self.object_pool.put_back(item_ticket, item);
            }
            return true;
        }
        false
    }

    pub(crate) fn run_script_update(&mut self) {
        for i in 0..self.object_pool.vec_len() {
            if let Some((item_ticket, mut item)) = self.object_pool.try_take_by_index(i) {
                if let Some(script) = &mut item.script {
                    let mut context = ScriptContext { hierarchy: self };
                    script.update(&mut context);
                }
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
