use imgui::{Ui, ImColor32};
use crate::{hierarchy::{Hierarchy, NodeExtension}, project_data::ProjectData, Selected};

const DS_SCREEN_X: u32 = 256;
const DS_SCREEN_Y: u32 = 192;
const CAM_OUTLINE_COLOUR: ImColor32 = ImColor32::from_rgba(100, 100, 100, 150);
const CAM_OUTLINE_THICKNESS: f32 = 2.0;
const COLLIDER_OUTLINE_COLOUR: ImColor32 = ImColor32::from_rgba(4, 217, 50, 200);
const COLLIDER_OUTLINE_THICKNESS: f32 = 2.0;
const SELECTED_OUTLINE_COLOUR: ImColor32 = ImColor32::from_rgba(13, 169, 252, 200);
const SELECTED_OUTLINE_THICKNESS: f32 = 2.0;
const BG_COLOUR: ImColor32 = ImColor32::from_rgb(30, 30, 30);
const GRID_LINE_DISTANCE: f32 = 50.0;
const GRID_LINE_COLOUR: ImColor32 = ImColor32::from_rgb(50, 50, 50);
const GRID_LINE_THICKNESS: f32 = 1.0;

pub struct WorldEditor {
    editor_cam_pos: [f32; 2]
}

impl WorldEditor {
    pub const fn new() -> Self {
        Self {
            editor_cam_pos: [GRID_LINE_DISTANCE, GRID_LINE_DISTANCE]
        }
    }

    pub fn draw_world_editor(&mut self, ui: &Ui, hierarchy: &mut Hierarchy, project_data: &mut ProjectData, selected: &mut Selected) {
        let _t = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
        ui.window("World")
            .build(|| {
                let draw_list = ui.get_window_draw_list();
                let canvas_pos = ui.cursor_screen_pos();
                let bottom_right = [canvas_pos[0] + ui.window_size()[0], canvas_pos[1] + ui.window_size()[1]];

                // Draw background colour
                draw_list.add_rect(canvas_pos, bottom_right, BG_COLOUR)
                    .filled(true)
                    .build();

                // Draw grid lines
                {
                    let mut cur_x = canvas_pos[0] + (self.editor_cam_pos[0] % GRID_LINE_DISTANCE);
                    while cur_x < bottom_right[0] {
                        draw_list.add_line([cur_x, canvas_pos[1]], [cur_x, bottom_right[1]], GRID_LINE_COLOUR)
                            .thickness(GRID_LINE_THICKNESS)
                            .build();
                        cur_x += GRID_LINE_DISTANCE;
                    }

                    let mut cur_y = canvas_pos[1] + (self.editor_cam_pos[1] % GRID_LINE_DISTANCE);
                    while cur_y < bottom_right[0] {
                        draw_list.add_line([canvas_pos[0], cur_y], [bottom_right[0], cur_y], GRID_LINE_COLOUR)
                            .thickness(GRID_LINE_THICKNESS)
                            .build();
                        cur_y += GRID_LINE_DISTANCE;
                    }
                }

                if ui.is_mouse_dragging(imgui::MouseButton::Right) {
                    let drag_delta = ui.io().mouse_delta;
                    self.editor_cam_pos = [self.editor_cam_pos[0] + drag_delta[0], self.editor_cam_pos[1] + drag_delta[1]];
                }

                let start_pos = [self.editor_cam_pos[0] + canvas_pos[0], self.editor_cam_pos[1] + canvas_pos[1]];
                Self::draw_node_recursive(ui, hierarchy, project_data, selected, 0, &draw_list, canvas_pos, start_pos);
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
        position: [f32; 2],
    ){
        if let Some(graph) = project_data.graphs.get(hierarchy.current_graph_idx) {
            if let Some(node) = graph.0.get(node_idx) {
                let node_canvas_pos = [node.transform.x.to_num::<i32>() as f32, node.transform.y.to_num::<i32>() as f32];
                let node_canvas_pos = [node_canvas_pos[0] + position[0], node_canvas_pos[1] + position[1]];
                let node_selected = matches!(hierarchy.selected_node_idx, Some(x) if usize::from(x) == node_idx);

                match &node.node_extension {
                    NodeExtension::Sprite(s) => {
                        if let Some(asset) = project_data.graphical_assets.get(&s.graphic_asset) {
                            let (width, height) = asset.size.to_dimensions();
                            let p_min = node_canvas_pos;
                            let p_max = [p_min[0] + width as f32, p_min[1] + height as f32];
                            draw_list.add_image(asset.texture.unwrap(), p_min, p_max).build();
                            if node_selected {
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
                        if node_selected {
                            draw_selected_rect_around(draw_list, top_left, bottom_right);
                        }
                    }
                    NodeExtension::RectCollider(c) => {
                        let top_left = node_canvas_pos;
                        let bottom_right = [top_left[0] + c.width.to_num::<f32>(), top_left[1] + c.height.to_num::<f32>()];
                        draw_list.add_rect(top_left, bottom_right, COLLIDER_OUTLINE_COLOUR)
                            .thickness(COLLIDER_OUTLINE_THICKNESS)
                            .build();
                        if node_selected {
                            draw_selected_rect_around(draw_list, top_left, bottom_right);
                        }
                    },
                    _ => {
                        if node_selected {
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
                        Self::draw_node_recursive(ui, hierarchy, project_data, selected, cur_child_idx_usize, draw_list, canvas_pos, node_canvas_pos);
                        cur_child_idx = match graph.0[cur_child_idx_usize].sibling_index {
                            Some(x) => x,
                            None => break,
                        };
                    }
                }
            }
        }
    }
}

fn draw_selected_rect_around(draw_list: &imgui::DrawListMut, top_left: [f32; 2], bottom_right: [f32; 2]) {
    let top_left = top_left.map(|x| x - SELECTED_OUTLINE_THICKNESS);
    let bottom_right = bottom_right.map(|x| x + SELECTED_OUTLINE_THICKNESS);
    draw_list.add_rect(top_left, bottom_right, SELECTED_OUTLINE_COLOUR)
        .thickness(SELECTED_OUTLINE_THICKNESS)
        .build();
}
