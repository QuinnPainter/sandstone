mod ui;
mod hierarchy;
mod project_loader;
mod project_builder;
mod project_data;
mod inspector;
mod files;
mod output_log;

use std::ffi::CString;

pub enum Selected {
    None,
    Node,
    File,
}

fn main() {
    log::set_boxed_logger(Box::new(output_log::Logger)).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let mut hierarchy_obj = hierarchy::Hierarchy::new();
    let mut proj_loader = project_loader::ProjectLoader::new();
    let mut project_data = project_data::ProjectData::new();
    let mut selected = Selected::None;
    
    let mut first_loop = true;

    let hierarchy_name = CString::new("Hierarchy").unwrap();
    let files_name = CString::new("Files").unwrap();
    let inspector_name = CString::new("Inspector").unwrap();
    let world_name = CString::new("World").unwrap();
    let graphs_name = CString::new("Graphs").unwrap();
    let log_name = CString::new("Log").unwrap();

    ui::mainloop(move |ui, exit| {
        //ui.show_demo_window(&mut true);
        // seems that imgui-rs has no abstractions for any docking stuff yet, so we must use the raw bindings
        unsafe {
            let view = imgui::sys::igGetMainViewport();
            let win_class = imgui::sys::ImGuiWindowClass_ImGuiWindowClass();
            let dockspace_id = imgui::sys::igDockSpaceOverViewport(view, 0, win_class);
            if first_loop {
                first_loop = false;

                imgui::sys::igDockBuilderRemoveNode(dockspace_id);
                imgui::sys::igDockBuilderAddNode(dockspace_id, imgui::sys::ImGuiDockNodeFlags_DockSpace);
                imgui::sys::igDockBuilderSetNodeSize(dockspace_id, (*view).Size);

                let mut tmp_id = dockspace_id;
                let dock_id_inspector = imgui::sys::igDockBuilderSplitNode(tmp_id,
                    imgui::sys::ImGuiDir_Right, 0.25,
                    std::ptr::null_mut::<u32>(), std::ptr::addr_of_mut!(tmp_id));
                let dock_id_files = imgui::sys::igDockBuilderSplitNode(tmp_id,
                    imgui::sys::ImGuiDir_Down, 0.20,
                    std::ptr::null_mut::<u32>(), std::ptr::addr_of_mut!(tmp_id));
                let dock_id_hierarchy = imgui::sys::igDockBuilderSplitNode(tmp_id,
                    imgui::sys::ImGuiDir_Left, 0.25,
                    std::ptr::null_mut::<u32>(), std::ptr::addr_of_mut!(tmp_id));

                imgui::sys::igDockBuilderDockWindow(hierarchy_name.as_ptr(), dock_id_hierarchy);
                imgui::sys::igDockBuilderDockWindow(graphs_name.as_ptr(), dock_id_hierarchy);
                imgui::sys::igDockBuilderDockWindow(files_name.as_ptr(), dock_id_files);
                imgui::sys::igDockBuilderDockWindow(log_name.as_ptr(), dock_id_files);
                imgui::sys::igDockBuilderDockWindow(inspector_name.as_ptr(), dock_id_inspector);
                imgui::sys::igDockBuilderDockWindow(world_name.as_ptr(), tmp_id);
                imgui::sys::igDockBuilderFinish(dockspace_id);
            }
        }
        ui.main_menu_bar(|| {
            ui.menu("File", || {
                if ui.menu_item("New") {
                    proj_loader.open_load_project_modal();
                }
                if ui.menu_item("Open") {
                    proj_loader.open_project_file_dialog();
                }
                // todo: open recent
                /*ui.menu("Open Recent", || {
                    ui.menu_item("Placeholder 1");
                    ui.menu_item("Placeholder 2");
                });*/
                // this Ctrl-S doesn't actually set up that shortcut, just displays the text
                if ui.menu_item_config("Save").shortcut("Ctrl+S").build() {
                    project_loader::save_project(&mut project_data);
                }
                //ui.menu_item("Save As..");
                ui.separator();
                if ui.menu_item_config("Quit").shortcut("Alt+F4").build() {
                    *exit = true;
                }
            });
            ui.menu("Run", || {
                if ui.menu_item("Build") {
                    project_builder::build(&mut project_data);
                }
                if ui.menu_item("Clean Build") {
                    project_builder::clean_build(&mut project_data);
                }
                //ui.menu_item("Build and Run")
            });
            if ui.menu_item("About") {}
        });

        project_data.check_file_scanner();
        proj_loader.update(ui, &mut project_data, &mut hierarchy_obj);

        inspector::draw_inspector(ui, &mut hierarchy_obj, &mut project_data, &mut selected);
        hierarchy_obj.draw_hierarchy(ui, &mut project_data, &mut selected);
        files::draw_files(ui, &mut project_data, &mut selected);
        output_log::draw_log(ui);
        ui.window("World")
            .build(|| {
                ui.text("someday this will work");
            });
    });
}
