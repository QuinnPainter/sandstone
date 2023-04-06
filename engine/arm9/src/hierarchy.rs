use core::num::NonZeroU32;
use alloc::{string::String, boxed::Box, vec::Vec};
use crate::{
    Script,
    ScriptContext,
    pool::{Pool, Handle},
    node::{Transform, Node, NodeScriptData, NodeExtensionHandle, NodeExtensionPools, sprite::SpriteExtensionHandler, camera::CameraExtensionHandler, rect_collider}
};

pub trait HierarchyPoolTrait<T> {
    fn borrow(&self, handle: Handle<T>) -> &T;
    fn try_borrow(&self, handle: Handle<T>) -> Option<&T>;
    fn borrow_mut(&mut self, handle: Handle<T>) -> &mut T;
    fn try_borrow_mut(&mut self, handle: Handle<T>) -> Option<&mut T>;
    fn handle_from_index(&self, index: usize) -> Option<Handle<T>>;
}

pub struct Hierarchy {
    pub root: Handle<Node>,
    pub(crate) object_pool: Pool<Node>,
    pub(crate) node_ext_pools: NodeExtensionPools,
    to_start_stack: Vec<Handle<Node>>,
    to_destroy_stack: Vec<Handle<Node>>,
    pub(crate) game_data: sandstone_common::SavedGameData,
    sprite_handler: SpriteExtensionHandler,
    camera_handler: CameraExtensionHandler,
    script_factory: fn(NonZeroU32) -> Box<dyn Script>,
}

macro hierarchy_pool_methods ($t:ty, $( $pool:ident ).+) {
    impl HierarchyPoolTrait<$t> for Hierarchy {
        #[inline]
        #[must_use]
        fn borrow(&self, handle: Handle<$t>) -> &$t {
            self.$($pool.)+borrow(handle)
        }

        #[inline]
        #[must_use]
        fn try_borrow(&self, handle: Handle<$t>) -> Option<&$t> {
            self.$($pool.)+try_borrow(handle)
        }
    
        #[inline]
        #[must_use]
        fn borrow_mut(&mut self, handle: Handle<$t>) -> &mut $t {
            self.$($pool.)+borrow_mut(handle)
        }
    
        #[inline]
        #[must_use]
        fn try_borrow_mut(&mut self, handle: Handle<$t>) -> Option<&mut $t> {
            self.$($pool.)+try_borrow_mut(handle)
        }
    
        #[inline]
        #[must_use]
        fn handle_from_index(&self, index: usize) -> Option<Handle<$t>> {
            self.$($pool.)+handle_from_index(index)
        }
    }
}

hierarchy_pool_methods!(Node, object_pool);
hierarchy_pool_methods!(crate::node::sprite::SpriteExtension, node_ext_pools.sprite_pool);
hierarchy_pool_methods!(crate::node::camera::CameraExtension, node_ext_pools.camera_pool);
hierarchy_pool_methods!(crate::node::rect_collider::RectColliderExtension, node_ext_pools.rect_collider_pool);

impl Hierarchy {
    pub fn new(game_data_raw: &[u8], script_factory: fn(NonZeroU32) -> Box<dyn Script>) -> Self {
        let mut object_pool: Pool<Node> = Pool::new();
        let root = object_pool.add(Node {
            child_handle: None,
            parent_handle: None,
            sibling_handle: None,
            name: String::new(),
            transform: Transform::default(),
            node_extension: NodeExtensionHandle::None,
            script_data: None,
            enabled: true,
            global_transform: Transform::default(),
            global_enabled: false,
        });

        Self {
            root,
            object_pool,
            node_ext_pools: NodeExtensionPools::new(),
            to_start_stack: Vec::new(),
            to_destroy_stack: Vec::new(),
            game_data: sandstone_common::deserialize(game_data_raw),
            sprite_handler: SpriteExtensionHandler::new(),
            camera_handler: CameraExtensionHandler::new(),
            script_factory,
        }
    }

    /// Shortcut to set_scene for the main scene.
    pub fn set_scene_main(&mut self) {
        // SAFETY: this isn't normally allowed because accessing self.game_data and set_scene
        // at the same time is 2 borrows. this is safe as long as set_scene doesn't modify game_data.main_graph
        // Is there a way to avoid this jank, without doing a clone?
        self.set_scene(unsafe {&*(self.game_data.main_graph.as_str() as *const str)});
    }

    /// Destroys the current scene, and starts the new one.
    /// As destroys are processed at the end of the frame, there will be a brief period where both scenes are loaded.
    pub fn set_scene(&mut self, name: &str) {
        if let Some(old_scene_root) = self.borrow(self.root).child_handle {
            self.destroy_node(old_scene_root);
        }
        self.spawn_object(name, self.root);
    }

    pub fn spawn_object(&mut self, graph_name: &str, parent: Handle<Node>) -> Handle<Node> {
        let saved_graph = self.game_data.graphs.get(graph_name)
            .unwrap_or_else(|| panic!("Tried to spawn invalid graph: {graph_name}"));

        // Push the nodes onto the object pool, with placeholder child, parent and sibling handles
        let new_handles: Vec<Handle<Node>> = saved_graph.nodes.iter().map(|node| {
            let handle = self.object_pool.add(Node {
                child_handle: None,
                parent_handle: None,
                sibling_handle: None,
                name: node.name.clone(),
                transform: Transform { x: node.transform.x, y: node.transform.y },
                node_extension: NodeExtensionHandle::None,
                script_data: node.script_type_id.map(|id| NodeScriptData {
                    type_id: id,
                    script: (self.script_factory)(id),
                }),
                enabled: node.enabled,
                global_transform: Transform::default(),
                global_enabled: false,
            });
            self.object_pool.borrow_mut(handle).node_extension =
                self.node_ext_pools.add_from_saved(handle, &node.node_extension);
            handle
        }).collect();
        
        // Wire up the child, parent and sibling handles for the new nodes
        let mut new_obj_root: Option<Handle<Node>> = None;
        for (snode, handle) in saved_graph.nodes.iter().zip(new_handles.iter()) {
            let node = self.object_pool.borrow_mut(*handle);
            node.child_handle = snode.child_index.map(|idx| new_handles[u32::from(idx) as usize]);
            node.sibling_handle = snode.sibling_index.map(|idx| new_handles[u32::from(idx) as usize]);
            node.parent_handle = snode.parent_index.map(|idx| new_handles[idx as usize]).or_else(|| {
                new_obj_root = Some(*handle);
                Some(parent)
            });
            self.to_start_stack.push(*handle);
        }
        let new_obj_root = new_obj_root.expect("Tried to create graph with no root node");
        self.link_new_child(parent, new_obj_root);
        new_obj_root
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
            ironds::nocash::print(&node.name);
            ironds::nocash::print(alloc::format!("Child: {:?}", node.child_handle).as_str());
            ironds::nocash::print(alloc::format!("Sibling: {:?}", node.sibling_handle).as_str());
            ironds::nocash::print(alloc::format!("Parent: {:?}", node.parent_handle).as_str());
            ironds::nocash::print("");
        }
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

    pub fn destroy_node(&mut self, handle: Handle<Node>) {
        self.to_destroy_stack.push(handle);
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
        for i in 0..self.object_pool.vec_len() {
            if let Some(handle) = self.handle_from_index(i) {
                let mut context = ScriptContext {
                    hierarchy: self,
                    handle,
                };
                // this could return None if an object was immediately destroyed after creating it
                let mut script_data = if let Some(item) = context.hierarchy.try_borrow_mut(handle) {
                    // return early - node is disabled
                    if item.global_enabled == false { continue; }
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
    }

    // Also updates the global "enabled" state.
    pub(crate) fn update_global_positions(&mut self) {
        let root = self.borrow(self.root);
        self.update_global_position_recursive(self.root, root.transform, root.enabled);
    }

    fn update_global_position_recursive(&mut self, handle: Handle<Node>, transform: Transform, enabled: bool) {
        let node = self.borrow_mut(handle);
        let new_enabled = node.enabled && enabled;
        let new_transform = Transform {
            x: node.transform.x + transform.x,
            y: node.transform.y + transform.y,
        };
        node.global_enabled = new_enabled;
        node.global_transform = new_transform;
        // Update child nodes recursively
        if let Some(mut cur_child_handle) = node.child_handle {
            loop {
                self.update_global_position_recursive(cur_child_handle, new_transform, new_enabled);
                cur_child_handle = match self.borrow(cur_child_handle).sibling_handle {
                    Some(x) => x,
                    None => break,
                };
            }
        }
    }

    pub(crate) fn run_extension_init(&mut self) {
        self.sprite_handler.sprite_init(&self.game_data);
    }

    pub(crate) fn run_extension_update(&mut self) {
        rect_collider::check_collisions(self);
        let cameras = self.camera_handler.get_active_cameras(self);
        self.sprite_handler.sprite_update(self, cameras);
    }

    pub(crate) fn process_pending_destroys(&mut self) {
        // unlink parent and sibling
        while let Some(root_handle) = self.to_destroy_stack.pop() {
            self.unlink_node(root_handle);
            let mut handle = root_handle;
            // Recursively delete children of node
            loop {
                let Some((_t, node)) = self.object_pool.try_take(handle) else {
                    panic!("Tried to destroy node with invalid handle");
                };
                self.node_ext_pools.destroy_extension(node.node_extension);
                handle = match if handle == root_handle {node.child_handle} else {node.sibling_handle} {
                    Some(x) => x,
                    None => break,
                };
            }
        }
    }

    fn unlink_node(&mut self, handle: Handle<Node>) {
        let node = self.object_pool.borrow(handle);
        let sibling_handle = node.sibling_handle;
        let parent_handle =
            node.parent_handle.unwrap_or_else(|| panic!("Tried to unlink root node"));
        let parent = self.object_pool.borrow_mut(parent_handle);
        if parent.child_handle.unwrap() == handle {
            parent.child_handle = sibling_handle;
        } else {
            self.loop_over_children(parent_handle, |node, _| {
                if node.sibling_handle == Some(handle) {
                    node.sibling_handle = sibling_handle;
                }
            });
        }
    }

    fn loop_over_children<F: FnMut(&mut Node, Handle<Node>)>(&mut self, handle: Handle<Node>, mut op: F) {
        if let Some(mut cur_child_handle) = self.object_pool.borrow(handle).child_handle {
            loop {
                let cur_child = self.object_pool.borrow_mut(cur_child_handle);
                op(cur_child, cur_child_handle);
                cur_child_handle = match cur_child.sibling_handle {
                    Some(x) => x,
                    None => break,
                }
            }
        }
    }
}

pub trait HasTypeId {
    fn type_id() -> NonZeroU32;
}
