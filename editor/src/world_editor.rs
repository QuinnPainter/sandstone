use imgui::Ui;
use crate::{hierarchy::{Hierarchy, NodeExtension}, project_data::ProjectData, Selected};

pub fn draw_world_editor(ui: &Ui, hierarchy: &mut Hierarchy, project_data: &mut ProjectData, selected: &mut Selected) {
    ui.window("World")
        .build(|| {
            let draw_list = ui.get_window_draw_list();
            let canvas_pos = ui.cursor_screen_pos();
            
            if let Some(graph) = project_data.graphs.get(hierarchy.current_graph_idx) {
                if let Some(h) = graph.0.get(1) {
                    match &h.node_extension {
                        NodeExtension::Sprite(s) => {
                            //println!("Showing {}", s.graphic_asset);
                            let asset = &project_data.graphical_assets[&s.graphic_asset];
                            let f: [f32; 2] = [canvas_pos[0] + 50.0, canvas_pos[1] + 50.0];
                            draw_list.add_image(asset.texture, canvas_pos, f).build();
                        },
                        _ => {},
                    }
                }
            }
            /*const RADIUS: f32 = 100.0;
            const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
            const F: [f32; 2] = [0.0, 0.0];
            let canvas_pos = ui.cursor_screen_pos();
            draw_list
                .add_line(
                    F,
                    [canvas_pos[0] + RADIUS, canvas_pos[1] + RADIUS],
                    RED,
                )
                .thickness(5.0)
                .build();*/

            //renderer.texture_map_mut()
            //imgui_glow_renderer::TextureMap
            //imgui::Texture
            //ui.tex
            //draw_list.add_image(texture_id, p_min, p_max)
            /*unsafe {
                let gl = renderer.gl_context();
                //renderer.texture_map_mut()
                let tex = gl.create_texture().unwrap();
                gl.bind_texture(glow::TEXTURE_2D, Some(tex));
                //gl.tex_image_2d(target, level, internal_format, width, height, border, format, ty, pixels)
                gl.delete_texture(tex);
            }*/
        });
}
