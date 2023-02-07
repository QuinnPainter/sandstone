use std::num::{NonZeroUsize, NonZeroU32};
use imgui::{Ui, TreeNodeFlags};
use stable_vec::StableVec;
use crate::ProjectData;

pub struct Transform {
    pub x: u32,
    pub y: u32
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0
        }
    }
}

pub struct Node {
    pub child_index: Option<NonZeroUsize>,
    pub parent_index: Option<usize>,
    pub sibling_index: Option<NonZeroUsize>,
    pub name: String,
    pub transform: Transform,
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
    new_graph_name_buffer: String
}

impl Hierarchy {
    pub fn new() -> Self {
        Self {
            current_graph_idx: 0,
            selected_node_idx: None,
            new_graph_name_buffer: String::new()
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
                                if graph.0.iter().find(|&(_, n)| n.name == node_name).is_none() {
                                    break;
                                }
                                node_number += 1;
                            }
                            let new_index = graph.0.push(Node {
                                child_index: None,
                                parent_index: Some(0), // parent is root node
                                sibling_index: None,
                                name: node_name,
                                transform: Transform::default(),
                                script_type_id: None,
                                enabled: true
                            });
                            let old_root_child = graph.0[0].child_index.replace(NonZeroUsize::new(new_index).unwrap());
                            graph.0[new_index].sibling_index = old_root_child;
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
                        script_type_id: None,
                        enabled: true
                    });
                    self.current_graph_idx = project_data.graphs.len();
                    self.selected_node_idx = None;
                    project_data.graphs.push(new_graph);
                    self.new_graph_name_buffer.clear();
                }
            });
    }

    fn draw_hierarchy_node(&mut self, ui: &Ui, project_data: &ProjectData, node_idx: usize) {
        if let Some(graph) = project_data.graphs.get(self.current_graph_idx) {
            if let Some(node) = graph.0.get(node_idx) {
                let mut tree_node_token: Option<imgui::TreeNodeToken> = None;
                // Root node is not drawn
                if node_idx != 0 {
                    let mut flags = TreeNodeFlags::empty();
                    flags.set(TreeNodeFlags::OPEN_ON_ARROW, true);
                    flags.set(TreeNodeFlags::LEAF, node.child_index.is_none());
                    // could change this to is_some_and if that gets stablised
                    // flags.set(TreeNodeFlags::SELECTED, selected_node_idx.is_some_and(|x| usize::from(x) == node_idx));
                    flags.set(TreeNodeFlags::SELECTED, matches!(self.selected_node_idx, Some(x) if usize::from(x) == node_idx));
                    
                    tree_node_token = ui.tree_node_config(format!("{}##TreeNode{}", node.name, node_idx).as_str()).flags(flags).push();
                    if ui.is_item_clicked() {
                        self.selected_node_idx = Some(NonZeroUsize::new(node_idx).unwrap());
                    }
                    if let Some(tooltip) = ui.drag_drop_source_config("HierarchyDragDrop").begin_payload(node_idx) {
                        // The tooltip displayed when dragging. This allocates every frame - could be improved?
                        ui.text(node.name.clone());
                        tooltip.end();
                    }
                    if let Some(target) = ui.drag_drop_target() {
                        let drag_drop_flags = imgui::DragDropFlags::empty();
                        if let Some(Ok(payload)) = target.accept_payload::<usize, _>("HierarchyDragDrop", drag_drop_flags) {
                            //set node_idx parent to payload.data
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
            let node_sibling_idx = (&graph.0[node_idx_usize]).sibling_index;
            let node_parent_idx = (&graph.0[node_idx_usize]).parent_index.unwrap();
            let node_parent = &mut graph.0[node_parent_idx];
            if node_parent.child_index.unwrap() == node_idx {
                node_parent.child_index = node_sibling_idx;
            } else {
                Hierarchy::loop_over_children(graph, node_parent_idx, |node: &mut Node| {
                    if node.sibling_index == Some(node_idx) {
                        node.sibling_index = node_sibling_idx;
                    }
                });
            }

            // Put all of the nodes children into a stack
            // optimisation todo: could consolidate these into one vec, and just increment an index instead of popping
            let mut to_delete_stack: Vec<usize> = vec![node_idx_usize];
            let mut tree_traverse_stack: Vec<usize> = vec![node_idx_usize];
            // loop until tree_traversal_stack is empty
            loop {
                let cur_node_idx = match tree_traverse_stack.pop() {
                    Some(x) => x,
                    None => break
                };
                let cur_node = &graph.0[cur_node_idx];
                // loop over immediate children
                if let Some(mut cur_child_idx) = cur_node.child_index {
                    loop {
                        let cur_child_idx_usize = usize::from(cur_child_idx);
                        tree_traverse_stack.push(cur_child_idx_usize);
                        to_delete_stack.push(cur_child_idx_usize);
                        cur_child_idx = match graph.0[cur_child_idx_usize].sibling_index {
                            Some(x) => x,
                            None => break
                        }
                    }
                }
            }

            // Delete all nodes in the stack
            for i in to_delete_stack {
                graph.0.remove(i);
            }
        }
    }

    fn loop_over_children<F: FnMut(&mut Node)>(graph: &mut NodeGraph, node_idx: usize, mut op: F) {
        let cur_node = &graph.0[node_idx];
        if let Some(mut cur_child_idx) = cur_node.child_index {
            loop {
                let cur_child_idx_usize = usize::from(cur_child_idx);
                op(&mut graph.0[cur_child_idx_usize]);
                cur_child_idx = match graph.0[cur_child_idx_usize].sibling_index {
                    Some(x) => x,
                    None => break
                }
            }
        }
    }
}
