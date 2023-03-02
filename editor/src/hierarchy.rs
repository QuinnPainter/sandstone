use std::num::{NonZeroUsize, NonZeroU32};
use imgui::{Ui, TreeNodeFlags};
use stable_vec::StableVec;
use crate::project_data::ProjectData;

#[derive(Default)]
pub struct Transform {
    pub x: u32,
    pub y: u32
}

pub enum NodeExtension {
    None,
    Sprite(SpriteExtension),
}

impl std::fmt::Display for NodeExtension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeExtension::None => write!(f, "None"),
            NodeExtension::Sprite(_) => write!(f, "Sprite"),
        }
    }
}

impl NodeExtension {
    pub fn from_saved(saved_extension: dsengine_common::SavedNodeExtension) -> Self {
        match saved_extension {
            dsengine_common::SavedNodeExtension::None => NodeExtension::None,
            dsengine_common::SavedNodeExtension::Sprite(s) => NodeExtension::Sprite(SpriteExtension {  }),
        }
    }

    pub fn to_saved(&self) -> dsengine_common::SavedNodeExtension {
        match self {
            NodeExtension::None => dsengine_common::SavedNodeExtension::None,
            NodeExtension::Sprite(_) => dsengine_common::SavedNodeExtension::Sprite(dsengine_common::SavedSpriteExtension{}),
        }
    }
}

pub struct SpriteExtension {}

pub struct Node {
    pub child_index: Option<NonZeroUsize>,
    pub parent_index: Option<usize>,
    pub sibling_index: Option<NonZeroUsize>,
    pub name: String,
    pub transform: Transform,
    pub node_extension: NodeExtension,
    pub script_type_id: Option<NonZeroU32>,
    pub enabled: bool
}

pub struct NodeGraph(pub StableVec<Node>);
impl NodeGraph {
    pub fn new() -> Self {
        Self (StableVec::new())
    }
}

pub struct Hierarchy {
    pub current_graph_idx: usize,
    pub selected_node_idx: Option<NonZeroUsize>,
    new_graph_name_buffer: String,
    pending_node_moves: Vec<NodeMove>
}

struct NodeMove {
    pub node_idx: usize,
    pub new_parent_idx: usize
}

impl Hierarchy {
    pub fn new() -> Self {
        Self {
            current_graph_idx: 0,
            selected_node_idx: None,
            new_graph_name_buffer: String::new(),
            pending_node_moves: Vec::new()
        }
    }

    pub fn draw_hierarchy(&mut self, ui: &Ui, project_data: &mut ProjectData) {
        ui.window("Hierarchy")
            .build(|| {
                if ui.is_window_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                    ui.open_popup("hierarchy_context");
                }
                if let Some(_p) = ui.begin_popup("hierarchy_context") {
                    if ui.selectable("Add Node") {
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
                                enabled: true
                            });
                            Hierarchy::link_node(graph, NonZeroUsize::new(new_index).unwrap(), 0);
                        }
                    }
                }
                self.draw_hierarchy_node(ui, project_data, 0);
            });
        ui.window("Graphs")
            .build(|| {
                for (i, g) in project_data.graphs.iter().enumerate() {
                    if let Some(root_node) = g.0.get(0) {
                        if ui.selectable_config(format!("{}##{}", &root_node.name, i))
                            .selected(self.current_graph_idx == i)
                            .build() {
                            self.current_graph_idx = i;
                            self.selected_node_idx = None;
                        }
                    }
                }
                ui.input_text("##new_graph_name", &mut self.new_graph_name_buffer).hint("New Graph").build();
                ui.same_line();
                if ui.button("+##add_graph") {
                    let mut new_graph = NodeGraph(StableVec::new());
                    new_graph.0.push(Node {
                        child_index: None,
                        parent_index: None,
                        sibling_index: None,
                        name: self.new_graph_name_buffer.clone(),
                        transform: Transform::default(),
                        node_extension: NodeExtension::None,
                        script_type_id: None,
                        enabled: true
                    });
                    self.current_graph_idx = project_data.graphs.len();
                    self.selected_node_idx = None;
                    project_data.graphs.push(new_graph);
                    self.new_graph_name_buffer.clear();
                }
            });

        // Process the pending node moves
        if let Some(graph) = project_data.graphs.get_mut(self.current_graph_idx) {
            while let Some(node_move) = self.pending_node_moves.pop() {
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
    }

    fn draw_hierarchy_node(&mut self, ui: &Ui, project_data: &ProjectData, node_idx: usize) {
        if let Some(graph) = project_data.graphs.get(self.current_graph_idx) {
            if let Some(node) = graph.0.get(node_idx) {
                let mut tree_node_token: Option<imgui::TreeNodeToken> = None;
                // Root node is not drawn
                if node_idx != 0 {
                    let mut flags = TreeNodeFlags::empty();
                    flags.set(TreeNodeFlags::OPEN_ON_ARROW, true);
                    flags.set(TreeNodeFlags::DEFAULT_OPEN, true);
                    flags.set(TreeNodeFlags::LEAF, node.child_index.is_none());
                    // could change this to is_some_and if that gets stablised
                    // flags.set(TreeNodeFlags::SELECTED, selected_node_idx.is_some_and(|x| usize::from(x) == node_idx));
                    flags.set(TreeNodeFlags::SELECTED, matches!(self.selected_node_idx, Some(x) if usize::from(x) == node_idx));
                    
                    tree_node_token = ui.tree_node_config(format!("{}##TreeNode{}", node.name, node_idx).as_str()).flags(flags).push();
                    if ui.is_item_clicked() {
                        self.selected_node_idx = Some(NonZeroUsize::new(node_idx).unwrap());
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
                if node_idx == 0 || tree_node_token.is_some() {
                    // Draw child nodes recursively
                    if let Some(mut cur_child_idx) = node.child_index {
                        loop {
                            let cur_child_idx_usize = usize::from(cur_child_idx);
                            self.draw_hierarchy_node(ui, project_data, cur_child_idx_usize);
                            cur_child_idx = match graph.0[cur_child_idx_usize].sibling_index {
                                Some(x) => x,
                                None => break
                            };
                        }
                    }
                }
                if let Some(t) = tree_node_token { t.pop(); }
            }
        }
    }

    pub fn delete_node(&mut self, project_data: &mut ProjectData, node_idx: NonZeroUsize) {
        // Deselect node just in case it is deleted
        self.selected_node_idx = None;

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
                    None => break
                }
            }
        }
    }
}
