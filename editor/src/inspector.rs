use imgui::Ui;
use crate::{hierarchy::{Hierarchy, NodeExtension, SpriteExtension, CameraExtension}, project_data::ProjectData, Selected};

pub fn draw_inspector(ui: &Ui, hierarchy: &mut Hierarchy, project_data: &mut ProjectData, selected: &mut Selected) {
    ui.window("Inspector")
        .build(|| {
            match *selected {
                Selected::None => {},
                Selected::File => { file_inspector(ui, project_data); },
                Selected::Node => { node_inspector(ui, hierarchy, project_data, selected); },
            }
        });
}

fn file_inspector(ui: &Ui, project_data: &mut ProjectData) {
    if let Some(selected_asset_name) = &mut project_data.selected_asset {
        if let Some(selected_asset) = project_data.graphical_assets.get_mut(selected_asset_name) {
            // Combo box for Size
            if let Some(_cb) = ui.begin_combo("Size", format!("{}", selected_asset.size)) {
                use sandstone_common::SpriteSize::*;
                let sizes = [_8x8, _16x16, _32x32, _64x64, _16x8, _32x8,
                                            _32x16, _64x32, _8x16, _8x32, _16x32, _32x64,];
                for s in sizes {
                    if ui.selectable(format!("{s}")) {
                        selected_asset.size = s;
                    }
                }
            }
        }
    }
}

fn node_inspector(ui: &Ui, hierarchy: &mut Hierarchy, project_data: &mut ProjectData, selected: &mut Selected) {
    if let Some(graph) = project_data.graphs.get_mut(hierarchy.current_graph_idx) {
        if let Some(selected_index) = hierarchy.selected_node_idx {
            let selected_node = &mut graph.0[selected_index.into()];
            ui.input_text("Name", &mut selected_node.name).build();
            ui.checkbox("Enabled", &mut selected_node.enabled);

            let mut pos: [f32; 2] = [selected_node.transform.x.to_num::<f32>(), selected_node.transform.y.to_num::<f32>()];
            imgui::Drag::new("Position").build_array(ui, &mut pos);
            selected_node.transform.x = fixed::types::I20F12::from_num(pos[0]);
            selected_node.transform.y = fixed::types::I20F12::from_num(pos[1]);

            if let Some(_cb) = ui.begin_combo("Extension", format!("{}", selected_node.node_extension)) {
                if ui.selectable("None") {
                    selected_node.node_extension = NodeExtension::None;
                }
                if ui.selectable("Sprite") {
                    selected_node.node_extension = NodeExtension::Sprite(SpriteExtension::default());
                }
                if ui.selectable("Camera") {
                    selected_node.node_extension = NodeExtension::Camera(CameraExtension::default());
                }
            }

            match &mut selected_node.node_extension {
                NodeExtension::None => (),
                NodeExtension::Sprite(s) => {
                    // Combo box for Graphic
                    if let Some(_cb) = ui.begin_combo("Graphic", &s.graphic_asset) {
                        for g in project_data.graphical_assets.keys() {
                            if ui.selectable(g) {
                                s.graphic_asset = g.clone();
                            }
                        }
                    }
                },
                NodeExtension::Camera(c) => {
                    ui.checkbox("Active for Main Engine", &mut c.active_main);
                    ui.checkbox("Active for Sub Engine", &mut c.active_sub);
                },
            }

            let mut script_id: u32 = selected_node.script_type_id.map_or(0, u32::from);
            ui.input_scalar("Script ID", &mut script_id).build();
            selected_node.script_type_id = std::num::NonZeroU32::new(script_id);

            if ui.button("Delete") {
                hierarchy.delete_node(project_data, selected, selected_index);
            }
        }
    }
}
