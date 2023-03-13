use imgui::Ui;
use crate::{hierarchy::{Hierarchy, NodeExtension}, project_data::ProjectData, Selected};

const DS_SCREEN_X: u32 = 256;
const DS_SCREEN_Y: u32 = 192;
const CAM_OUTLINE_COLOUR: imgui::ImColor32 = imgui::ImColor32::from_rgba(100, 100, 100, 150);
const CAM_OUTLINE_THICKNESS: f32 = 2.0;
const SELECTED_OUTLINE_COLOUR: imgui::ImColor32 = imgui::ImColor32::from_rgba(13, 169, 252, 200);
const SELECTED_OUTLINE_THICKNESS: f32 = 2.0;

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
            let node_canvas_pos = world_to_canvas_pos(canvas_pos, (node.transform.x, node.transform.y));
            match &node.node_extension {
                NodeExtension::Sprite(s) => {
                    if let Some(asset) = project_data.graphical_assets.get(&s.graphic_asset) {
                        let (width, height) = asset.size.to_dimensions();
                        let p_min = node_canvas_pos;
                        let p_max = [p_min[0] + width as f32, p_min[1] + height as f32];
                        draw_list.add_image(asset.texture.unwrap(), p_min, p_max).build();
                        if matches!(hierarchy.selected_node_idx, Some(x) if usize::from(x) == node_idx) {
                            draw_selected_rect_around(draw_list, p_min, p_max);
                        }
                    }
                },
                NodeExtension::Camera(_) => {
                    let top_left = node_canvas_pos;
                    let bottom_right = [top_left[0] + DS_SCREEN_X as f32, top_left[1] + DS_SCREEN_Y as f32];
                    draw_list.add_rect(top_left, bottom_right, CAM_OUTLINE_COLOUR)
                        .thickness(CAM_OUTLINE_THICKNESS)
                        .build();
                    if matches!(hierarchy.selected_node_idx, Some(x) if usize::from(x) == node_idx) {
                        draw_selected_rect_around(draw_list, top_left, bottom_right);
                    }
                }
                _ => {
                    if matches!(hierarchy.selected_node_idx, Some(x) if usize::from(x) == node_idx) {
                        draw_list.add_circle(node_canvas_pos, 2.0, SELECTED_OUTLINE_COLOUR)
                            .filled(true)
                            .build();
                    }
                },
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

fn world_to_canvas_pos(canvas_pos: [f32; 2], (x, y): (fixed::types::I20F12, fixed::types::I20F12)) ->[f32; 2] {
    [canvas_pos[0] + x.to_num::<i32>() as f32, canvas_pos[1] + y.to_num::<i32>() as f32]
}

fn draw_selected_rect_around(draw_list: &imgui::DrawListMut, top_left: [f32; 2], bottom_right: [f32; 2]) {
    let top_left = top_left.map(|x| x - SELECTED_OUTLINE_THICKNESS);
    let bottom_right = bottom_right.map(|x| x + SELECTED_OUTLINE_THICKNESS);
    draw_list.add_rect(top_left, bottom_right, SELECTED_OUTLINE_COLOUR)
        .thickness(SELECTED_OUTLINE_THICKNESS)
        .build();
}
