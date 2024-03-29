use imgui::Ui;

use crate::{project_data::ProjectData, Selected};

pub fn draw_files(ui: &Ui, project_data: &mut ProjectData, selected: &mut Selected) {
    ui.window("Files")
        .build(|| {
            for (asset_name, _) in project_data.graphical_assets.iter() {
                if ui.selectable(asset_name) {
                    *selected = Selected::File(asset_name.clone());
                }
            }
        });
}
