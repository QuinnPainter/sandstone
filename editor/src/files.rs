use imgui::Ui;

use crate::{project_data::ProjectData, Selected};

pub fn draw_files(ui: &Ui, project_data: &mut ProjectData, selected: &mut Selected) {
    ui.window("Files")
        .build(|| {
            for (asset_name, _) in project_data.graphical_assets.iter() {
                //ui.text(&asset_name);
                if ui.selectable(&asset_name) {
                    *selected = Selected::File;
                    project_data.selected_asset = Some(asset_name.clone());
                }
            }
        });
}
