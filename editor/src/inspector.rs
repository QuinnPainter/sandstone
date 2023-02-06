use imgui::Ui;
use crate::{hierarchy::Hierarchy, ProjectData};

pub fn draw_inspector(ui: &Ui, hierarchy: &mut Hierarchy, project_data: &mut ProjectData) {
    ui.window("Inspector")
        .build(|| {
            if let Some(graph) = project_data.graphs.get_mut(hierarchy.current_graph_idx) {
                if let Some(selected_index) = hierarchy.selected_node_idx {
                    let selected_node = &mut graph.0[selected_index.into()];
                    if ui.input_text("Name", &mut selected_node.name).build() {
                        // called when the field is edited
                        dbg!(&selected_node.name);
                    }
                    if ui.checkbox("Enabled", &mut selected_node.enabled) {}
                    /*unsafe {
                        //if ui.input_float("X", &mut pos_x).build() {}
                        //if ui.input_float2("Y", &mut pos_y).build() {}
                        let pos_label = std::ffi::CString::new("Position").unwrap();
                        imgui::sys::igDragFloat2(pos_label.as_ptr(), &mut pos as *mut ImVec2 as *mut f32,
                            0.1, 0.0, 0.0, std::ptr::null(), 0);
                    }*/
                }
            }
        });
}
