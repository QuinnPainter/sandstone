mod ui;
mod hierarchy;
mod project_loader;
mod project_builder;
mod project_data;
mod inspector;
mod files;
mod image_helper;
mod output_log;
mod world_editor;

use std::ffi::CString;
use std::sync::{Arc, Mutex};
use std::thread;

pub enum Selected {
    None,
    Node(usize),
    File(String),
    Graph(usize),
}

fn main() {
    log::set_boxed_logger(Box::new(output_log::Logger)).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    let mut hierarchy_obj = hierarchy::Hierarchy::new();
    let mut proj_loader = project_loader::ProjectLoader::new();
    let project_data = Arc::new(Mutex::new(project_data::ProjectData::new()));
    let mut world_editor = world_editor::WorldEditor::new();
    let mut selected = Selected::None;
    let mut building_frames = 0;
    
    let mut first_loop = true;

    let hierarchy_name = CString::new("Hierarchy").unwrap();
    let files_name = CString::new("Files").unwrap();
    let inspector_name = CString::new("Inspector").unwrap();
    let world_name = CString::new("World").unwrap();
    let graphs_name = CString::new("Graphs").unwrap();
    let log_name = CString::new("Log").unwrap();

    ui::mainloop(move |ui, renderer, exit| {
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
                let mut dock_id_hierarchy = imgui::sys::igDockBuilderSplitNode(tmp_id,
                    imgui::sys::ImGuiDir_Left, 0.25,
                    std::ptr::null_mut::<u32>(), std::ptr::addr_of_mut!(tmp_id));
                let dock_id_log = imgui::sys::igDockBuilderSplitNode(tmp_id,
                    imgui::sys::ImGuiDir_Down, 0.25,
                    std::ptr::null_mut::<u32>(), std::ptr::addr_of_mut!(tmp_id));
                let dock_id_files = imgui::sys::igDockBuilderSplitNode(dock_id_hierarchy,
                    imgui::sys::ImGuiDir_Down, 0.30,
                    std::ptr::null_mut::<u32>(), std::ptr::addr_of_mut!(dock_id_hierarchy));

                imgui::sys::igDockBuilderDockWindow(hierarchy_name.as_ptr(), dock_id_hierarchy);
                imgui::sys::igDockBuilderDockWindow(graphs_name.as_ptr(), dock_id_hierarchy);
                imgui::sys::igDockBuilderDockWindow(files_name.as_ptr(), dock_id_files);
                imgui::sys::igDockBuilderDockWindow(log_name.as_ptr(), dock_id_log);
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
                // this Ctrl-S doesn't actually set up that shortcut, just displays the text
                if ui.menu_item_config("Save").shortcut("Ctrl+S").build() {
                    project_loader::save_project(&mut project_data.lock().unwrap());
                }
                ui.separator();
                if ui.menu_item_config("Quit").shortcut("Alt+F4").build() {
                    *exit = true;
                }
            });
            ui.menu("Run", || {
                let mut build = false;
                let mut clean = false;
                if ui.menu_item("Build") {
                    build = true;
                }
                if ui.menu_item("Clean Build") {
                    build = true;
                    clean = true;
                }
                if build {
                    let p_data = project_data.clone();
                    thread::spawn(move || {
                        let mut project_data = p_data.lock().unwrap();
                        if clean {
                            project_builder::clean_build(&mut project_data);
                        } else {
                            project_builder::build(&mut project_data);
                        }
                    });
                    building_frames = 0;
                }
            });
            if ui.menu_item("About") {}
        });

        if let Ok(mut project_data) = project_data.try_lock() {
            project_data.check_file_scanner(renderer);
            proj_loader.update(ui, &mut project_data, &mut hierarchy_obj, renderer, &mut selected);

            inspector::draw_inspector(ui, &mut hierarchy_obj, &mut project_data, &mut selected);
            hierarchy_obj.draw_hierarchy(ui, &mut project_data, &mut selected);
            files::draw_files(ui, &mut project_data, &mut selected);
            world_editor.draw_world_editor(ui, &mut hierarchy_obj, &mut project_data, &mut selected);
        } else {
            // Bouncing back and forth animation
            const LOAD_BAR_WIDTH: usize = 30;
            building_frames += 1;
            let anim_pos = (building_frames / 3) % (LOAD_BAR_WIDTH * 2);
            let (before, after) = if anim_pos < LOAD_BAR_WIDTH {
                (anim_pos, LOAD_BAR_WIDTH - anim_pos)
            } else {
                (LOAD_BAR_WIDTH - (anim_pos - LOAD_BAR_WIDTH), anim_pos - LOAD_BAR_WIDTH)
            };
            let build_text =
                String::from("[")
                + &" ".repeat(before)
                + "<=>"
                + &" ".repeat(after)
                + "]";
            ui.text("Building...");
            ui.text(build_text);
        }
        output_log::draw_log(ui);
    });
}
