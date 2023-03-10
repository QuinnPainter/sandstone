use imgui::Ui;
use crate::{hierarchy::{Hierarchy, NodeExtension}, project_data::ProjectData, Selected};

pub fn draw_world_editor(ui: &Ui, hierarchy: &mut Hierarchy, project_data: &mut ProjectData, selected: &mut Selected) {
    ui.window("World")
        .build(|| {
            let draw_list = ui.get_window_draw_list();
            let canvas_pos = ui.cursor_screen_pos();

            draw_node_recursive(ui, hierarchy, project_data, selected, 0, &draw_list, canvas_pos)
        });
}

// todo: this recursive node logic is duplicated in Hierarchy. how to deduplicate?
fn draw_node_recursive(
    ui: &Ui,
    hierarchy: &mut Hierarchy,
    project_data: &ProjectData,
    selected: &mut Selected,
    node_idx: usize,
    draw_list: &imgui::DrawListMut,
    canvas_pos: [f32; 2],
){
    if let Some(graph) = project_data.graphs.get(hierarchy.current_graph_idx) {
        if let Some(node) = graph.0.get(node_idx) {
            match &node.node_extension {
                NodeExtension::Sprite(s) => {
                    if let Some(asset) = project_data.graphical_assets.get(&s.graphic_asset) {
                        let (width, height) = asset.size.to_dimensions();
                        let p_min = [canvas_pos[0] + node.transform.x.to_num::<u32>() as f32, canvas_pos[1] + node.transform.y.to_num::<u32>() as f32];
                        let p_max = [p_min[0] + width as f32, p_min[1] + height as f32];
                        draw_list.add_image(asset.texture.unwrap(), p_min, p_max).build();
                    }
                },
                _ => {},
            }

            // Draw child nodes recursively
            if let Some(mut cur_child_idx) = node.child_index {
                loop {
                    let cur_child_idx_usize = usize::from(cur_child_idx);
                    draw_node_recursive(ui, hierarchy, project_data, selected, cur_child_idx_usize, draw_list, canvas_pos);
                    cur_child_idx = match graph.0[cur_child_idx_usize].sibling_index {
                        Some(x) => x,
                        None => break,
                    };
                }
            }
        }
    }
}
