use core::num::NonZeroU32;
use alloc::{string::String, boxed::Box, vec::Vec};
use crate::{
    Script,
    ScriptContext,
    pool::{Pool, Handle},
    node::{Transform, Node, NodeScriptData, NodeExtensionHandle, NodeExtensionPools}
};
use sandstone_common::SavedPrefabs;

pub struct Hierarchy {
    pub root: Handle<Node>,
    object_pool: Pool<Node>,
    node_ext_pools: NodeExtensionPools,
    to_start_stack: Vec<Handle<Node>>,
    saved_prefab_data: SavedPrefabs,
}

impl Hierarchy {
    pub fn new() -> Self {
        let mut object_pool: Pool<Node> = Pool::new();
        let root = object_pool.add(Node {
            child_handle: None,
            parent_handle: None,
            sibling_handle: None,
            name: String::new(),
            transform: Transform::default(),
            node_extension: NodeExtensionHandle::None,
            script_data: None,
            enabled: true
        });

        Self {
            root,
            object_pool,
            node_ext_pools: NodeExtensionPools::new(),
            to_start_stack: Vec::new(),
            saved_prefab_data: sandstone_common::deserialize(unsafe { PREFAB_DATA.unwrap() })
        }
    }

    pub fn spawn_prefab(&mut self, index: u32, parent: Handle<Node>) {
        let saved_graph = self.saved_prefab_data.graphs.get(index as usize)
            .unwrap_or_else(|| panic!("Tried to spawn invalid prefab index: {index}"));
        let mut new_handles: Vec<Handle<Node>> = Vec::new();

        // Push the nodes onto the object pool, with placeholder child, parent and sibling handles
        for node in &saved_graph.nodes {
            let handle = self.object_pool.add(Node {
                child_handle: None,
                parent_handle: None,
                sibling_handle: None,
                name: node.name.clone(),
                transform: Transform { x: node.transform.x, y: node.transform.y },
                node_extension: NodeExtensionHandle::None,
                script_data: node.script_type_id.map(|id| NodeScriptData {
                    type_id: id,
                    script: run_script_factory(id)
                }),
                enabled: node.enabled
            });
            self.object_pool.borrow_mut(handle).node_extension =
                NodeExtensionHandle::from_saved(&mut self.node_ext_pools, handle, &node.node_extension);
            new_handles.push(handle);
        }
        
        // Wire up the child, parent and sibling handles for the new nodes
        let mut prefab_root: Option<Handle<Node>> = None;
        for (snode, handle) in saved_graph.nodes.iter().zip(new_handles.iter()) {
            let node = self.object_pool.borrow_mut(*handle);
            node.child_handle = snode.child_index.map(|idx| new_handles[u32::from(idx) as usize]);
            node.sibling_handle = snode.sibling_index.map(|idx| new_handles[u32::from(idx) as usize]);
            node.parent_handle = snode.parent_index.map(|idx| new_handles[idx as usize]).or({
                prefab_root = Some(*handle);
                Some(parent)
            });
            self.to_start_stack.push(*handle);
        }
        self.link_new_child(parent, prefab_root.expect("Tried to create prefab with no root node"));

    }

    /*pub fn add(&mut self, mut item: Node, parent: Handle<Node>) {
        item.parent_handle = Some(parent);
        let handle = self.object_pool.add(item);
        self.link_new_child(parent, handle);

        self.to_start_stack.push(handle);
    }*/

    fn link_new_child(&mut self, parent: Handle<Node>, child: Handle<Node>) {
        let parent_obj = self.object_pool.borrow_mut(parent);
        self.object_pool.borrow_mut(child).sibling_handle = parent_obj.child_handle.replace(child);
    }

    pub fn pretty_print_hierarchy_structure(&self) {
        for node in &self.object_pool {
            ironds::nocash::print("Node");
            ironds::nocash::print(alloc::format!("Child: {:?}", node.child_handle).as_str());
            ironds::nocash::print(alloc::format!("Sibling: {:?}", node.sibling_handle).as_str());
            ironds::nocash::print(alloc::format!("Parent: {:?}", node.parent_handle).as_str());
        }
    }
    
    #[inline(always)]
    #[must_use]
    pub fn vec_len(&self) -> usize {
        self.object_pool.vec_len()
    }

    #[inline]
    #[must_use]
    pub fn borrow(&self, handle: Handle<Node>) -> &Node {
        self.object_pool.borrow(handle)
    }

    #[inline]
    #[must_use]
    pub fn try_borrow(&self, handle: Handle<Node>) -> Option<&Node> {
        self.object_pool.try_borrow(handle)
    }

    #[inline]
    #[must_use]
    pub fn borrow_mut(&mut self, handle: Handle<Node>) -> &mut Node {
        self.object_pool.borrow_mut(handle)
    }

    #[inline]
    #[must_use]
    pub fn try_borrow_mut(&mut self, handle: Handle<Node>) -> Option<&mut Node> {
        self.object_pool.try_borrow_mut(handle)
    }

    #[inline]
    #[must_use]
    pub fn handle_from_index(&self, index: usize) -> Handle<Node> {
        self.object_pool.handle_from_index(index)
    }

    // todo: recursive search?
    // could have fast path for situation where search root is graph root node
    // as we can iterate over vec sequentially instead of following the tree

    #[must_use]
    pub fn find_by_name(&mut self, search_root: Handle<Node>, name: &str) -> Option<Handle<Node>> {
        self.find(search_root, |x| x.name == name)
    }

    #[must_use]
    pub fn find_by_script_type<T>(&mut self, search_root: Handle<Node>) -> Option<Handle<Node>>
    where T: Script + HasTypeId {
        self.find(search_root, |x| {
            match &x.script_data {
                Some(s_data) => s_data.type_id == <T as HasTypeId>::type_id(),
                None => false
            }
        })
    }

    #[must_use]
    pub fn find<P>(&mut self, search_root: Handle<Node>, mut predicate: P) -> Option<Handle<Node>>
    where P: FnMut(&Node) -> bool, {
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
            let mut context = ScriptContext {
                hierarchy: self,
                handle,
            };
            // this could return None if an object was immediately destroyed after creating it
            let mut script_data = if let Some(item) = context.hierarchy.try_borrow_mut(handle) {
                if let Some(script_data) = item.script_data.take() {
                    script_data
                } else {
                    return true; // return early - item has no Script
                }
            } else {
                return true; // return early - invalid handle on start stack (should panic here?)
            };
            script_data.script.start(&mut context);

            // put script back
            if let Some(item) = context.hierarchy.try_borrow_mut(handle) {
                item.script_data = Some(script_data);
            }
            return true;
        }
        false
    }

    pub(crate) fn run_script_update(&mut self) {
        for i in 0..self.vec_len() {
            let handle = self.handle_from_index(i);
            let mut context = ScriptContext {
                hierarchy: self,
                handle,
            };
            // this could return None if an object was immediately destroyed after creating it
            let mut script_data = if let Some(item) = context.hierarchy.try_borrow_mut(handle) {
                if let Some(script_data) = item.script_data.take() {
                    script_data
                } else {
                    continue; // return early - item has no Script
                }
            } else {
                continue; // return early - invalid handle (should panic here?)
            };
            script_data.script.update(&mut context);

            // put script back
            if let Some(item) = context.hierarchy.try_borrow_mut(handle) {
                item.script_data = Some(script_data);
            }
        }
    }

    pub(crate) fn run_extension_update(&mut self) {
        crate::node::sprite::sprite_update(&self.object_pool, &self.node_ext_pools);
    }
}

static mut PREFAB_DATA: Option<&[u8]> = None;

pub fn init_prefab_data(data: &'static [u8]) {
    unsafe { PREFAB_DATA = Some(data); }
}

static mut SCRIPT_FACTORY: Option<fn(NonZeroU32) -> Box<dyn Script>> = None;

pub fn init_script_factory(f: fn(NonZeroU32) -> Box<dyn Script>) {
    unsafe { SCRIPT_FACTORY = Some(f); }
}

#[inline]
#[must_use]
pub (crate) fn run_script_factory(id: NonZeroU32) -> Box<dyn Script> {
    unsafe {
        debug_assert_ne!(SCRIPT_FACTORY, None, "Cannot use Script Factory before initialisation!");
        SCRIPT_FACTORY.unwrap_unchecked()(id)
    }
}

pub trait HasTypeId {
    fn type_id() -> NonZeroU32;
}
