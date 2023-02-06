use std::num::{NonZeroUsize, NonZeroU32};
use imgui::{Ui, TreeNodeFlags};
use stable_vec::StableVec;
use crate::ProjectData;

pub struct Transform {
    pub x: u32,
    pub y: u32
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
    pub selected_node_idx: Option<NonZeroUsize>
}

impl Hierarchy {
    pub fn new() -> Self {
        Self {
            current_graph_idx: 0,
            selected_node_idx: None
        }
    }

    pub fn draw_hierarchy(&mut self, ui: &Ui, project_data: &mut ProjectData) {
        ui.window("Hierarchy")
            .build(|| {
                if ui.is_window_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                    ui.open_popup("hierarchy_context");
                }
                if let Some(_p) = ui.begin_popup("hierarchy_context") {
                    if ui.selectable("Add Stuff") {}
                    if ui.selectable("Things") {}
                }
                self.draw_hierarchy_node(ui, project_data, 0);
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
}
