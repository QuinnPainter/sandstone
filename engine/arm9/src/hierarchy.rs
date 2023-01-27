use core::num::NonZeroU32;
use alloc::{string::String, boxed::Box, vec::Vec};
use crate::{Script, pool::{Pool, Handle}, ScriptContext};
use dsengine_common::SavedPrefabs;

#[derive(Eq, PartialEq, Clone, Copy, Default, Debug)]
pub struct Transform {
    pub x: u32,
    pub y: u32,
}

pub struct NodeScriptData {
    pub type_id: NonZeroU32,
    pub script: Box<dyn Script>
}

pub struct Node {
    pub child_handle: Option<Handle<Node>>, // todo: maybe could be more efficient, could just be Index without Generation
    pub parent_handle: Option<Handle<Node>>, // should only be None on root node
    pub sibling_handle: Option<Handle<Node>>,
    pub name: String,
    pub transform: Transform,
    pub script_data: Option<NodeScriptData>,
    pub enabled: bool,
}

impl Node {
    pub fn cast_script<T>(&self) -> &T
    where T: Script + HasTypeId {
        let s_data = self.script_data.as_ref().expect("Tried to cast_script on an object which has no Script");
        assert_eq!(<T as HasTypeId>::type_id(), s_data.type_id, "Tried to cast_script with mismatching types");
        unsafe { &*(s_data.script.as_ref() as *const dyn Script as *const T) }
    }

    pub fn cast_script_mut<T>(&mut self) -> &mut T
    where T: Script + HasTypeId {
        let s_data = self.script_data.as_mut().expect("Tried to cast_script on an object which has no Script");
        assert_eq!(<T as HasTypeId>::type_id(), s_data.type_id, "Tried to cast_script with mismatching types");
        unsafe { &mut *(s_data.script.as_mut() as *mut dyn Script as *mut T) }
    }
}

pub struct Hierarchy {
    pub root: Handle<Node>,
    object_pool: Pool<Node>,
    to_start_stack: Vec<Handle<Node>>,
    saved_prefab_data: SavedPrefabs
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
            script_data: None,
            enabled: true
        });

        Self {
            root,
            object_pool,
            to_start_stack: Vec::new(),
            saved_prefab_data: dsengine_common::deserialize_prefabs(unsafe { PREFAB_DATA.unwrap() })
        }
    }

    pub fn spawn_prefab(&mut self, index: u32, parent: Handle<Node>) {
        /*let mut saved_graph = dsengine_common::SavedNodeGraph {nodes: Vec::new()};
        saved_graph.nodes.push(dsengine_common::SavedNode {
            child_index: None,
            sibling_index: None,
            name: String::from("derp"),
            transform: dsengine_common::SavedTransform { x: 0, y: 0 },
            script_type_id: None,
            enabled: true
        });
        let a = dsengine_common::do_serialize(&saved_graph);
        ironds::nocash::print(&alloc::format!("{:?}", a).to_string());*/

        let saved_graph = self.saved_prefab_data.0.get(index as usize).expect("Tried to spawn invalid prefab index");

        let mut new_handles: Vec<Handle<Node>> = Vec::new();

        // Push the nodes onto the object pool, with placeholder child, parent and sibling handles
        for node in &saved_graph.nodes {
            new_handles.push(self.object_pool.add(Node {
                child_handle: None,
                parent_handle: None,
                sibling_handle: None,
                name: node.name.clone(),
                transform: Transform { x: node.transform.x, y: node.transform.y },
                script_data: match node.script_type_id {
                    Some(id) => Some(NodeScriptData {
                        type_id: id,
                        script: run_script_factory(id)
                    }),
                    None => None
                },
                enabled: node.enabled
            }));
        }
        
        // Wire up the child, parent and sibling handles for the new nodes
        let mut prefab_root: Option<Handle<Node>> = None;
        for (snode, handle) in (&saved_graph.nodes).iter().zip(new_handles.iter()) {
            let node = self.object_pool.borrow_mut(*handle);
            let idx_to_handle = |opt_index: Option<NonZeroU32>| -> Option<Handle<Node>> {
                match opt_index {
                    Some(idx) => Some(new_handles[(u32::from(idx)-1) as usize]),
                    None => None
                }
            };
            node.child_handle = idx_to_handle(snode.child_index);
            node.sibling_handle = idx_to_handle(snode.sibling_index);
            node.parent_handle = idx_to_handle(snode.sibling_index).or({
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

    #[inline]
    #[must_use]
    pub fn borrow(&self, handle: Handle<Node>) -> &Node {
        self.object_pool.borrow(handle)
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
            // this could return None if an object was immediately destroyed after creating it
            if let Some((item_ticket, mut item)) = self.object_pool.try_take(handle) {
                if let Some(script_data) = &mut item.script_data {
                    let mut context = ScriptContext { hierarchy: self };
                    script_data.script.start(&mut context);
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
                if let Some(script_data) = &mut item.script_data {
                    let mut context = ScriptContext { hierarchy: self };
                    script_data.script.update(&mut context);
                }
                self.object_pool.put_back(item_ticket, item);
            }
        }
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
