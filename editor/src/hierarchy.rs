use std::num::{NonZeroUsize, NonZeroU32};
use imgui::{Ui, TreeNodeFlags};
use stable_vec::StableVec;
use crate::{project_data::ProjectData, Selected};

#[derive(Default, Clone, Copy, Debug)]
pub struct Transform {
    pub x: fixed::types::I20F12,
    pub y: fixed::types::I20F12,
}

#[derive(Debug)]
pub enum NodeExtension {
    None,
    Sprite(SpriteExtension),
    Camera(CameraExtension),
    RectCollider(RectColliderExtension),
}

impl std::fmt::Display for NodeExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeExtension::None => write!(f, "None"),
            NodeExtension::Sprite(_) => write!(f, "Sprite"),
            NodeExtension::Camera(_) => write!(f, "Camera"),
            NodeExtension::RectCollider(_) => write!(f, "Rect Collider"),
        }
    }
}

impl NodeExtension {
    pub fn from_saved(saved_extension: sandstone_common::SavedNodeExtension) -> Self {
        match saved_extension {
            sandstone_common::SavedNodeExtension::None => NodeExtension::None,
            sandstone_common::SavedNodeExtension::Sprite(s) => NodeExtension::Sprite(SpriteExtension {
                graphic_asset: s.graphic_asset,
                sprite_type: match s.sprite_type {
                    sandstone_common::SavedSpriteType::Normal => SpriteType::Normal,
                    sandstone_common::SavedSpriteType::Affine(a) => SpriteType::Affine(AffineSpriteData {
                        rotation: a.rotation,
                        scale_x: a.scale_x,
                        scale_y: a.scale_y,
                    }),
                },
            }),
            sandstone_common::SavedNodeExtension::Camera(c) => NodeExtension::Camera(CameraExtension { active_main: c.active_main, active_sub: c.active_sub }),
            sandstone_common::SavedNodeExtension::RectCollider(c) => NodeExtension::RectCollider(RectColliderExtension { width: c.width, height: c.height })
        }
    }

    pub fn to_saved(&self) -> sandstone_common::SavedNodeExtension {
        match self {
            NodeExtension::None => sandstone_common::SavedNodeExtension::None,
            NodeExtension::Sprite(s) => sandstone_common::SavedNodeExtension::Sprite(sandstone_common::SavedSpriteExtension {
                graphic_asset: s.graphic_asset.clone(),
                sprite_type: match s.sprite_type {
                    SpriteType::Normal => sandstone_common::SavedSpriteType::Normal,
                    SpriteType::Affine(a) => sandstone_common::SavedSpriteType::Affine(sandstone_common::SavedAffineSpriteData {
                        rotation: a.rotation,
                        scale_x: a.scale_x,
                        scale_y: a.scale_y,
                    }),
                },
            }),
            NodeExtension::Camera(c) => sandstone_common::SavedNodeExtension::Camera(sandstone_common::SavedCameraExtension { active_main: c.active_main, active_sub: c.active_sub }),
            NodeExtension::RectCollider(c) => sandstone_common::SavedNodeExtension::RectCollider(sandstone_common::SavedRectColliderExtension { width: c.width, height: c.height }),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AffineSpriteData {
    pub rotation: fixed::types::I20F12,
    pub scale_x: fixed::types::I20F12,
    pub scale_y: fixed::types::I20F12,
}

impl Default for AffineSpriteData {
    fn default() -> Self {
        Self {
            rotation: fixed::types::I20F12::lit("0"),
            scale_x: fixed::types::I20F12::lit("1"),
            scale_y: fixed::types::I20F12::lit("1"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub enum SpriteType {
    #[default]
    Normal,
    Affine(AffineSpriteData),
}

#[derive(Default, Debug)]
pub struct SpriteExtension {
    pub graphic_asset: String,
    pub sprite_type: SpriteType,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct CameraExtension {
    pub active_main: bool,
    pub active_sub: bool,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct RectColliderExtension {
    pub width: fixed::types::I20F12,
    pub height: fixed::types::I20F12,
}

#[derive(Debug)]
pub struct Node {
    pub child_index: Option<NonZeroUsize>,
    pub parent_index: Option<usize>,
    pub sibling_index: Option<NonZeroUsize>,
    pub name: String,
    pub transform: Transform,
    pub node_extension: NodeExtension,
    pub script_type_id: Option<NonZeroU32>,
    pub enabled: bool,
}

pub struct NodeGraph(pub StableVec<Node>);
impl NodeGraph {
    pub fn new() -> Self {
        Self (StableVec::new())
    }
}

pub struct Hierarchy {
    pub current_graph_idx: usize,
    new_graph_name_buffer: String,
    pending_node_moves: Vec<NodeMove>,
}

#[derive(Clone, Copy, Debug)]
struct NodeMove {
    pub node_idx: usize,
    pub new_parent_idx: usize
}

impl Hierarchy {
    pub fn new() -> Self {
        Self {
            current_graph_idx: 0,
            new_graph_name_buffer: String::new(),
            pending_node_moves: Vec::new(),
        }
    }

    pub fn draw_hierarchy(&mut self, ui: &Ui, project_data: &mut ProjectData, selected: &mut Selected) {
        ui.window("Hierarchy")
            .build(|| {
                if ui.is_window_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                    ui.open_popup("hierarchy_context");
                }
                if let Some(_p) = ui.begin_popup("hierarchy_context") {
                    if ui.selectable("Add Node") {
                        self.add_node(project_data, selected);
                    }
                }
                self.draw_hierarchy_node(ui, project_data, selected, 0);
            });
        ui.window("Graphs")
            .build(|| {
                for (i, g) in project_data.graphs.iter().enumerate() {
                    if let Some(root_node) = g.0.get(0) {
                        if ui.selectable_config(format!("{}##{}", &root_node.name, i))
                            .selected(self.current_graph_idx == i)
                            .build() {
                            self.current_graph_idx = i;
                            *selected = Selected::Graph(i);
                        }
                    }
                }
                ui.input_text("##new_graph_name", &mut self.new_graph_name_buffer).hint("New Graph").build();
                ui.same_line();
                if ui.button("+##add_graph") {
                    self.add_graph(project_data, selected);
                }
            });

        // Process the pending node moves
        while let Some(node_move) = self.pending_node_moves.pop() {
            self.move_node(project_data, node_move);
        }
    }

    fn draw_hierarchy_node(&mut self, ui: &Ui, project_data: &ProjectData, selected: &mut Selected, node_idx: usize) {
        if let Some(graph) = project_data.graphs.get(self.current_graph_idx) {
            if let Some(node) = graph.0.get(node_idx) {
                let tree_node_token;
                // Draw node
                {
                    let mut flags = TreeNodeFlags::empty();
                    flags.set(TreeNodeFlags::OPEN_ON_ARROW, true);
                    flags.set(TreeNodeFlags::DEFAULT_OPEN, true);
                    flags.set(TreeNodeFlags::LEAF, node.child_index.is_none());
                    // could change this to is_some_and if that gets stablised
                    // flags.set(TreeNodeFlags::SELECTED, selected_node_idx.is_some_and(|x| usize::from(x) == node_idx));
                    flags.set(TreeNodeFlags::SELECTED, matches!(selected, &mut Selected::Node(x) if x == node_idx));
                    
                    tree_node_token = ui.tree_node_config(format!("{}##TreeNode{}", node.name, node_idx).as_str()).flags(flags).push();
                    if ui.is_item_clicked() {
                        *selected = Selected::Node(node_idx);
                    }
                    if let Some(tooltip) = ui.drag_drop_source_config("HierarchyDragDrop").begin_payload(node_idx) {
                        // The tooltip displayed when dragging
                        ui.text(&node.name);
                        tooltip.end();
                    }
                    if let Some(target) = ui.drag_drop_target() {
                        let drag_drop_flags = imgui::DragDropFlags::empty();
                        if let Some(Ok(payload)) = target.accept_payload::<usize, _>("HierarchyDragDrop", drag_drop_flags) {
                            self.pending_node_moves.push(NodeMove { node_idx: payload.data, new_parent_idx: node_idx });
                        }
                        target.pop();
                    }
                }
                if let Some(t) = tree_node_token {
                    // Draw child nodes recursively
                    if let Some(mut cur_child_idx) = node.child_index {
                        loop {
                            let cur_child_idx_usize = usize::from(cur_child_idx);
                            self.draw_hierarchy_node(ui, project_data, selected, cur_child_idx_usize);
                            cur_child_idx = match graph.0[cur_child_idx_usize].sibling_index {
                                Some(x) => x,
                                None => break,
                            };
                        }
                    }
                    t.pop();
                }
            }
        }
    }

    fn add_graph(&mut self, project_data: &mut ProjectData, selected: &mut Selected) {
        let mut new_graph = NodeGraph::new();
        new_graph.0.push(Node {
            child_index: None,
            parent_index: None,
            sibling_index: None,
            name: self.new_graph_name_buffer.clone(),
            transform: Transform::default(),
            node_extension: NodeExtension::None,
            script_type_id: None,
            enabled: true,
        });
        self.current_graph_idx = project_data.graphs.len();
        // If this is the first graph created, make it the Main Graph
        if project_data.graphs.is_empty() {
            project_data.main_graph = Some(self.current_graph_idx as u32);
        }
        *selected = Selected::Graph(self.current_graph_idx);
        project_data.graphs.push(new_graph);
        self.new_graph_name_buffer.clear();
    }

    fn add_node(&mut self, project_data: &mut ProjectData, selected: &mut Selected) {
        if let Some(graph) = project_data.graphs.get_mut(self.current_graph_idx) {
            let mut node_number = 0;
            let mut node_name;
            loop {
                node_name = format!("Node {node_number}");
                if !graph.0.iter().any(|(_, n)| n.name == node_name) {
                    break;
                }
                node_number += 1;
            }
            let new_index = graph.0.push(Node {
                child_index: None,
                parent_index: None,
                sibling_index: None,
                name: node_name,
                transform: Transform::default(),
                node_extension: NodeExtension::None,
                script_type_id: None,
                enabled: true,
            });
            Hierarchy::link_node(graph, NonZeroUsize::new(new_index).unwrap(), 0);
            *selected = Selected::Node(new_index);
        }
    }

    fn move_node(&mut self, project_data: &mut ProjectData, node_move: NodeMove) {
        if let Some(graph) = project_data.graphs.get_mut(self.current_graph_idx) {
            let mut valid_move = true;

            // check if the new parent is the same node that is being moved
            if node_move.node_idx == node_move.new_parent_idx {
                valid_move = false;
            }

            // check if the new parent is actually a child of the node
            Hierarchy::loop_over_children_recursive(graph, node_move.node_idx, |_, idx| {
                if idx == node_move.new_parent_idx {
                    valid_move = false;
                }
            });

            if valid_move {
                Hierarchy::unlink_node(graph, NonZeroUsize::new(node_move.node_idx).unwrap());
                Hierarchy::link_node(graph, NonZeroUsize::new(node_move.node_idx).unwrap(), node_move.new_parent_idx);
            }
        }
    }

    pub fn delete_node(&mut self, project_data: &mut ProjectData, selected: &mut Selected, node_idx: NonZeroUsize) {
        // Deselect node just in case it is deleted
        *selected = Selected::None;

        let node_idx_usize = usize::from(node_idx);
        if let Some(graph) = project_data.graphs.get_mut(self.current_graph_idx) {
            // Unlink node from the graph
            Hierarchy::unlink_node(graph, node_idx);

            // Put all of the nodes children into a stack
            let mut to_delete_stack: Vec<usize> = vec![node_idx_usize];
            Hierarchy::loop_over_children_recursive(graph, node_idx_usize, |_, idx| {
                to_delete_stack.push(idx);
            });

            // Delete all nodes in the stack
            for i in to_delete_stack {
                graph.0.remove(i);
            }
        }
    }
    
    fn unlink_node(graph: &mut NodeGraph, node_idx: NonZeroUsize) {
        let node_idx_usize = usize::from(node_idx);
        let node_sibling_idx = graph.0[node_idx_usize].sibling_index;
        let node_parent_idx = graph.0[node_idx_usize].parent_index.unwrap();
        let node_parent = &mut graph.0[node_parent_idx];
        if node_parent.child_index.unwrap() == node_idx {
            node_parent.child_index = node_sibling_idx;
        } else {
            Hierarchy::loop_over_children(graph, node_parent_idx, |node, _| {
                if node.sibling_index == Some(node_idx) {
                    node.sibling_index = node_sibling_idx;
                }
            });
        }
    }

    fn link_node(graph: &mut NodeGraph, node_idx: NonZeroUsize, parent: usize) {
        let old_root_child = graph.0[parent].child_index.replace(node_idx);
        let node = &mut graph.0[usize::from(node_idx)];
        node.sibling_index = old_root_child;
        node.parent_index = Some(parent);
    }

    fn loop_over_children_recursive<F: FnMut(&mut Node, usize)>(graph: &mut NodeGraph, node_idx: usize, mut op: F) {
        let mut tree_traverse_stack: Vec<usize> = vec![node_idx];
        // loop until tree_traverse_stack is empty
        while let Some(cur_node_idx) = tree_traverse_stack.pop() {
            // loop over immediate children
            Hierarchy::loop_over_children(graph, cur_node_idx, |node, idx| {
                tree_traverse_stack.push(idx);
                op(node, idx);
            });
        }
    }

    fn loop_over_children<F: FnMut(&mut Node, usize)>(graph: &mut NodeGraph, node_idx: usize, mut op: F) {
        let cur_node = &graph.0[node_idx];
        if let Some(mut cur_child_idx) = cur_node.child_index {
            loop {
                let cur_child_idx_usize = usize::from(cur_child_idx);
                op(&mut graph.0[cur_child_idx_usize], cur_child_idx_usize);
                cur_child_idx = match graph.0[cur_child_idx_usize].sibling_index {
                    Some(x) => x,
                    None => break,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_graph() {
        const TEST_GRAPH_NAME: &str = "TestGraph";
        let mut h = Hierarchy::new();
        let mut project_data = ProjectData::new();
        let mut selected = Selected::None;
        h.new_graph_name_buffer = TEST_GRAPH_NAME.to_string();
        h.add_graph(&mut project_data, &mut selected);
        assert_eq!(project_data.graphs.len(), 1);
        assert_eq!(project_data.graphs[0].0[0].name, TEST_GRAPH_NAME);
    }

    #[test]
    fn add_node() {
        let mut h = Hierarchy::new();
        let mut project_data = ProjectData::new();
        let mut selected = Selected::None;
        h.add_graph(&mut project_data, &mut selected);
        h.add_node(&mut project_data, &mut selected);
        assert_eq!(project_data.graphs[0].0.num_elements(), 2);
        assert_eq!(project_data.graphs[0].0[1].name, "Node 0");
    }

    #[test]
    fn move_node() {
        let (mut h, mut project_data, mut selected) = (Hierarchy::new(), ProjectData::new(), Selected::None);
        h.add_graph(&mut project_data, &mut selected);
        h.add_node(&mut project_data, &mut selected);
        h.add_node(&mut project_data, &mut selected);
        h.add_node(&mut project_data, &mut selected);
        h.move_node(&mut project_data, NodeMove { node_idx: 2, new_parent_idx: 1 });
        let graph = &mut project_data.graphs[0].0;
        assert_eq!(graph[1].child_index, Some(NonZeroUsize::new(2).unwrap()));
        assert_eq!(graph[2].parent_index, Some(1));
        h.move_node(&mut project_data, NodeMove { node_idx: 3, new_parent_idx: 1 });
        let graph = &mut project_data.graphs[0].0;
        assert_eq!(graph[1].child_index, Some(NonZeroUsize::new(3).unwrap()));
        assert_eq!(graph[3].sibling_index, Some(NonZeroUsize::new(2).unwrap()));
        assert_eq!(graph[3].parent_index, Some(1));
        assert_eq!(graph[2].parent_index, Some(1));
    }

    #[test]
    fn delete_node() {
        let (mut h, mut project_data, mut selected) = (Hierarchy::new(), ProjectData::new(), Selected::None);
        h.add_graph(&mut project_data, &mut selected);
        h.add_node(&mut project_data, &mut selected);
        h.add_node(&mut project_data, &mut selected);
        h.add_node(&mut project_data, &mut selected);

        h.move_node(&mut project_data, NodeMove { node_idx: 3, new_parent_idx: 2 });
        h.delete_node(&mut project_data, &mut selected, NonZeroUsize::new(2).unwrap());
        assert_eq!(project_data.graphs[0].0.num_elements(), 2);
    }
}

