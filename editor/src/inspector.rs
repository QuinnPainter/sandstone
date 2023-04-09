use imgui::Ui;
use crate::{hierarchy::{Hierarchy, NodeExtension, SpriteExtension, CameraExtension, RectColliderExtension, SpriteType, AffineSpriteData}, project_data::ProjectData, Selected};

pub fn draw_inspector(ui: &Ui, hierarchy: &mut Hierarchy, project_data: &mut ProjectData, selected: &mut Selected) {
    ui.window("Inspector")
        .build(|| {
            match *selected {
                Selected::None => {},
                Selected::File(_) => { file_inspector(ui, project_data, selected); },
                Selected::Node(_) => { node_inspector(ui, hierarchy, project_data, selected); },
                Selected::Graph(_) => { graph_inspector(ui, project_data, selected); },
            }
        });
}

fn file_inspector(ui: &Ui, project_data: &mut ProjectData, selected: &mut Selected) {
    let Selected::File(selected_asset_name) = selected else { return; };
    let Some(selected_asset) = project_data.graphical_assets.get_mut(selected_asset_name) else { return; };
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

fn node_inspector(ui: &Ui, hierarchy: &mut Hierarchy, project_data: &mut ProjectData, selected: &mut Selected) {
    let &mut Selected::Node(selected_index) = selected else { return; };
    let Some(graph) = project_data.graphs.get_mut(hierarchy.current_graph_idx)  else { return; };
    let selected_node = &mut graph.0[selected_index];
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
        if ui.selectable("Rect Collider") {
            selected_node.node_extension = NodeExtension::RectCollider(RectColliderExtension::default());
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

            let mut affine = !matches!(s.sprite_type, SpriteType::Normal);
            if ui.checkbox("Affine Sprite", &mut affine) {
                if affine {
                    s.sprite_type = SpriteType::Affine(AffineSpriteData::default());
                } else {
                    s.sprite_type = SpriteType::Normal;
                }
            }
            if let SpriteType::Affine(a) = &mut s.sprite_type {
                // Rotation input
                let mut rotation: f32 = a.rotation.to_num::<f32>();
                imgui::Drag::new("Rotation")
                    .range(0.0, fixed::types::I20F12::MAX.to_num::<f32>())
                    .build(ui, &mut rotation);
                a.rotation = fixed::types::I20F12::from_num(rotation);
                // Scale input
                let mut scale: [f32; 2] = [a.scale_x.to_num::<f32>(), a.scale_y.to_num::<f32>()];
                imgui::Drag::new("Scale")
                    .range(0.0, fixed::types::I20F12::MAX.to_num::<f32>())
                    .build_array(ui, &mut scale);
                a.scale_x = fixed::types::I20F12::from_num(scale[0]);
                a.scale_y = fixed::types::I20F12::from_num(scale[1]);
            }
        },
        NodeExtension::Camera(c) => {
            ui.checkbox("Active for Main Engine", &mut c.active_main);
            ui.checkbox("Active for Sub Engine", &mut c.active_sub);
        },
        NodeExtension::RectCollider(c) => {
            let mut dims: [f32; 2] = [c.width.to_num::<f32>(), c.height.to_num::<f32>()];
            imgui::Drag::new("Size")
                .range(0.0, fixed::types::I20F12::MAX.to_num::<f32>())
                .build_array(ui, &mut dims);
            c.width = fixed::types::I20F12::from_num(dims[0]);
            c.height = fixed::types::I20F12::from_num(dims[1]);
        },
    }

    let mut script_id: u32 = selected_node.script_type_id.map_or(0, u32::from);
    ui.input_scalar("Script ID", &mut script_id).build();
    selected_node.script_type_id = std::num::NonZeroU32::new(script_id);

    // Root node cannot be deleted
    if let Some(selected_index) = std::num::NonZeroUsize::new(selected_index) {
        if ui.button("Delete") {
            hierarchy.delete_node(project_data, selected, selected_index);
        }
    }
}

fn graph_inspector(ui: &Ui, project_data: &mut ProjectData, selected: &mut Selected) {
    let &mut Selected::Graph(selected_index) = selected else { return; };
    let Some(node) = project_data.graphs[selected_index].0.get_mut(0) else { return; };
    ui.input_text("Name", &mut node.name).build();
    if project_data.main_graph == Some(selected_index as u32) {
        ui.text("This is the Main Graph");
    } else if ui.button("Set as Main Graph") {
        project_data.main_graph = Some(selected_index as u32);
    }
}
