use std::num::{NonZeroUsize, NonZeroU32};
use dsengine_common::SavedNode;
use imgui::Ui;

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
                draw_hierarchy_node(ui, &self.graph, 0, &mut self.selected_node_idx);
            });
    }
}

// this function is quite a mess. wonder if it can be cleaned up and made into a method of Hierarchy
fn draw_hierarchy_node(ui: &Ui, graph: &Vec<dsengine_common::SavedNode>, node_idx: usize, selected_node_idx: &mut Option<NonZeroUsize>) {
    if let Some(node) = graph.get(node_idx) {
        fn draw_child_nodes (ui: &Ui, graph: &Vec<dsengine_common::SavedNode>, selected_node_idx: &mut Option<NonZeroUsize>, node: &SavedNode) {
            if let Some(mut cur_child_idx) = node.child_index {
                loop {
                    let cur_child_idx_usize = u32::from(cur_child_idx) as usize;
                    draw_hierarchy_node(ui, graph, cur_child_idx_usize, selected_node_idx);
                    cur_child_idx = match graph[cur_child_idx_usize].sibling_index {
                        Some(x) => x,
                        None => break
                    }
                }
            }
        }

        // Root node is not drawn
        if node_idx == 0 {
            draw_child_nodes(ui, graph, selected_node_idx, node);
        } else {
            let mut flags = imgui::TreeNodeFlags::empty();
            flags |= imgui::TreeNodeFlags::OPEN_ON_ARROW;
            if node.child_index.is_none() {
                flags |= imgui::TreeNodeFlags::LEAF;
            }
            // could change this to is_some_and if that gets stablised
            // if selected_node_idx.is_some_and(|x| x == node_idx) {
            if matches!(selected_node_idx, Some(x) if usize::from(*x) == node_idx) {
                flags |= imgui::TreeNodeFlags::SELECTED;
            }

            fn inner_node_logic (ui: &Ui, node_idx: usize, selected_node_idx: &mut Option<NonZeroUsize>) {
                if ui.is_item_clicked() {
                    *selected_node_idx = Some(NonZeroUsize::new(node_idx).unwrap());
                }
                if let Some(tooltip) = ui.drag_drop_source_config("HierarchyDragDrop").begin_payload(node_idx) {
                    ui.text("derp");
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

            if ui.tree_node_config(format!("{}##TreeNode{}", node.name, node_idx).as_str()).flags(flags).build(|| {
                inner_node_logic(ui, node_idx, selected_node_idx);
                draw_child_nodes(ui, graph, selected_node_idx, node);
            }).is_none() {
                inner_node_logic(ui, node_idx, selected_node_idx);
            }
        }
    }
}
