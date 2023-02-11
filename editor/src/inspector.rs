use imgui::Ui;
use crate::{hierarchy::Hierarchy, project_data::ProjectData};

pub fn draw_inspector(ui: &Ui, hierarchy: &mut Hierarchy, project_data: &mut ProjectData) {
    ui.window("Inspector")
        .build(|| {
            if let Some(graph) = project_data.graphs.get_mut(hierarchy.current_graph_idx) {
                if let Some(selected_index) = hierarchy.selected_node_idx {
                    let selected_node = &mut graph.0[selected_index.into()];
                    ui.input_text("Name", &mut selected_node.name).build();
                    ui.checkbox("Enabled", &mut selected_node.enabled);
                    let mut pos: [u32; 2] = [selected_node.transform.x, selected_node.transform.y];
                    imgui::Drag::new("Position").build_array(ui, &mut pos);
                    selected_node.transform.x = pos[0];
                    selected_node.transform.y = pos[1];
                    let mut script_id: u32 = selected_node.script_type_id.map_or(0, u32::from);
                    ui.input_scalar("Script ID", &mut script_id).build();
                    selected_node.script_type_id = std::num::NonZeroU32::new(script_id);
                    if ui.button("Delete") {
                        hierarchy.delete_node(project_data, selected_index);
                    }
                }
            }
        });
}
