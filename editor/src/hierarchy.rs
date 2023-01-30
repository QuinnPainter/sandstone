use std::num::{NonZeroUsize, NonZeroU32};
use dsengine_common::SavedNode;
use imgui::{Ui, TreeNodeFlags};

pub struct Hierarchy {
    graph: Vec<SavedNode>,
    selected_node_idx: Option<NonZeroUsize>
}

impl Hierarchy {
    pub fn new() -> Self {
        let mut graph: Vec<SavedNode> = Vec::new();
        graph.push(dsengine_common::SavedNode {
            child_index: Some(NonZeroU32::new(1).unwrap()),
            sibling_index: None,
            name: String::from("__EDITOR_ROOT__"),
            transform: dsengine_common::SavedTransform { x: 0, y: 0 },
            script_type_id: None,
            enabled: true
        });
        graph.push(dsengine_common::SavedNode {
            child_index: None,
            sibling_index: Some(NonZeroU32::new(2).unwrap()),
            name: String::from("derp"),
            transform: dsengine_common::SavedTransform { x: 0, y: 0 },
            script_type_id: Some(core::num::NonZeroU32::new(1).unwrap()),
            enabled: true
        });
        graph.push(dsengine_common::SavedNode {
            child_index: Some(NonZeroU32::new(3).unwrap()),
            sibling_index: None,
            name: String::from("herp"),
            transform: dsengine_common::SavedTransform { x: 0, y: 0 },
            script_type_id: Some(core::num::NonZeroU32::new(1).unwrap()),
            enabled: true
        });
        graph.push(dsengine_common::SavedNode {
            child_index: None,
            sibling_index: Some(NonZeroU32::new(4).unwrap()),
            name: String::from("flerp"),
            transform: dsengine_common::SavedTransform { x: 0, y: 0 },
            script_type_id: Some(core::num::NonZeroU32::new(1).unwrap()),
            enabled: true
        });
        graph.push(dsengine_common::SavedNode {
            child_index: None,
            sibling_index: None,
            name: String::from("merp"),
            transform: dsengine_common::SavedTransform { x: 0, y: 0 },
            script_type_id: Some(core::num::NonZeroU32::new(1).unwrap()),
            enabled: true
        });
        Self {
            graph,
            selected_node_idx: None
        }
    }

    pub fn draw_hierarchy(&mut self, ui: &Ui) {
        ui.window("Hierarchy")
            .build(|| {
                if ui.is_window_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                    ui.open_popup("hierarchy_context");
                }
                if let Some(_p) = ui.begin_popup("hierarchy_context") {
                    if ui.selectable("Add Stuff") {}
                    if ui.selectable("Things") {}
                }
                self.draw_hierarchy_node(ui, 0);
            });
    }

    fn draw_hierarchy_node(&mut self, ui: &Ui, node_idx: usize) {
        if let Some(node) = self.graph.get(node_idx) {
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
                        let cur_child_idx_usize = u32::from(cur_child_idx) as usize;
                        self.draw_hierarchy_node(ui, cur_child_idx_usize);
                        cur_child_idx = match self.graph[cur_child_idx_usize].sibling_index {
                            Some(x) => x,
                            None => break
                        }
                    }
                }
            }
            if let Some(t) = tree_node_token { t.pop(); }
        }
    }    
}
