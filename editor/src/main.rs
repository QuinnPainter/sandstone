mod ui;

use imgui::sys::ImVec2;
use std::ffi::CString;
//use std::fs::File;
//use std::io::Write;
use std::thread;

mod hierarchy;

fn main() {
    /*let mut saved_graph = dsengine_common::SavedNodeGraph {nodes: Vec::new()};
    saved_graph.nodes.push(dsengine_common::SavedNode {
        child_index: None,
        sibling_index: None,
        name: String::from("derp"),
        transform: dsengine_common::SavedTransform { x: 0, y: 0 },
        script_type_id: Some(core::num::NonZeroU32::new(1).unwrap()),
        enabled: true
    });
    let mut prefabs = dsengine_common::SavedPrefabs(Vec::new());
    prefabs.0.push(saved_graph);
    let mut saved_graph = dsengine_common::SavedNodeGraph {nodes: Vec::new()};
    saved_graph.nodes.push(dsengine_common::SavedNode {
        child_index: None,
        sibling_index: None,
        name: String::from("flerp"),
        transform: dsengine_common::SavedTransform { x: 0, y: 0 },
        script_type_id: Some(core::num::NonZeroU32::new(2).unwrap()),
        enabled: true
    });
    prefabs.0.push(saved_graph);

    let a = dsengine_common::serialize_prefabs(&prefabs);
    {
        let mut prefab_file = File::create("../test/prefab_data.bin").unwrap();
        prefab_file.write_all(&a).unwrap();
    }
    println!("{:?}", a);*/

    let mut hierarchy_obj = hierarchy::Hierarchy::new();
    
    let mut first_loop = true;
    let mut name = String::from("garf");
    //let mut pos_x = 0.0f32;
    //let mut pos_y = 0.0f32;
    let mut enabled = true;
    let mut pos = ImVec2::new(0.0, 0.0);

    let hierarchy_name = CString::new("Hierarchy").unwrap();
    let files_name = CString::new("Files").unwrap();
    let inspector_name = CString::new("Inspector").unwrap();
    let world_name = CString::new("World").unwrap();

    let pos_label = CString::new("Position").unwrap();

    let mut new_project_path_buffer = String::new();
    let mut new_project_name_buffer = String::new();

    let (file_dialog_transmitter, file_dialog_receiver) = std::sync::mpsc::channel::<FileDialogReturnInfo>();

    ui::mainloop(move |ui, exit| {
        //ui.show_demo_window(&mut true);
        let mut open_load_project_dialog = false;
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
                    imgui::sys::ImGuiDir_Right, 0.20,
                    std::ptr::null::<u32>() as *mut u32, &mut tmp_id as *mut u32);
                let dock_id_files = imgui::sys::igDockBuilderSplitNode(tmp_id,
                    imgui::sys::ImGuiDir_Down, 0.20,
                    std::ptr::null::<u32>() as *mut u32, &mut tmp_id as *mut u32);
                let dock_id_hierarchy = imgui::sys::igDockBuilderSplitNode(tmp_id,
                    imgui::sys::ImGuiDir_Left, 0.20,
                    std::ptr::null::<u32>() as *mut u32, &mut tmp_id as *mut u32);

                imgui::sys::igDockBuilderDockWindow(hierarchy_name.as_ptr(), dock_id_hierarchy);
                imgui::sys::igDockBuilderDockWindow(files_name.as_ptr(), dock_id_files);
                imgui::sys::igDockBuilderDockWindow(inspector_name.as_ptr(), dock_id_inspector);
                imgui::sys::igDockBuilderDockWindow(world_name.as_ptr(), tmp_id);
                imgui::sys::igDockBuilderFinish(dockspace_id);
                open_load_project_dialog = true;
            }
        }
        ui.modal_popup_config("Load Project").resizable(false).always_auto_resize(true).build(|| {
            if let Some(tab_bar_token) = ui.tab_bar_with_flags("tabs", imgui::TabBarFlags::empty()) {
                if let Some(tab_token) = ui.tab_item("New") {
                    ui.text("Project name");
                    ui.input_text("##ProjectName", &mut new_project_name_buffer)
                        .callback(imgui::InputTextCallback::CHAR_FILTER, FileNameInputFilter).build();
                    ui.text("Location");
                    ui.input_text("##PathInput", &mut new_project_path_buffer).build();
                    ui.same_line();
                    if ui.button("Browse") {
                        new_project(file_dialog_transmitter.clone());
                    }
                    ui.spacing();
                    // Enable text wrapping using the current window width
                    let text_wrap_token = ui.push_text_wrap_pos();
                    ui.text_disabled(format!("Project will be created in {}",
                        std::path::Path::new(&new_project_path_buffer).join(&new_project_name_buffer).display()));
                    text_wrap_token.end();
                    ui.spacing();
                    if ui.button("Create") {
                        ui.close_current_popup();
                    }
                    tab_token.end();
                }
                if let Some(tab_token) = ui.tab_item("Open") {
                    if ui.button_with_size("Open", [90.0, 30.0]) {
                        open_project(file_dialog_transmitter.clone());
                    }
                    tab_token.end();
                }
                tab_bar_token.end();
            }
        });
        ui.main_menu_bar(|| {
            ui.menu("File", || {
                if ui.menu_item("New") {
                    open_load_project_dialog = true;
                }
                if ui.menu_item("Open") {
                    open_project(file_dialog_transmitter.clone());
                }
                // todo: open recent
                /*ui.menu("Open Recent", || {
                    ui.menu_item("Placeholder 1");
                    ui.menu_item("Placeholder 2");
                });*/
                // this Ctrl-S doesn't actually set up that shortcut, just displays the text
                ui.menu_item_config("Save").shortcut("Ctrl+S").build();
                //ui.menu_item("Save As..");
                ui.separator();
                if ui.menu_item_config("Quit").shortcut("Alt+F4").build() {
                    *exit = true;
                }
            });
            if ui.menu_item("About") {}
        });

        // Workaround for https://github.com/ocornut/imgui/issues/331
        if open_load_project_dialog {
            new_project_path_buffer = String::from(
                home::home_dir().unwrap_or(std::path::PathBuf::new())
                .to_str().unwrap_or(""));
            ui.open_popup("Load Project");
        }

        // Handle the file dialog close event
        if let Ok(dialog_info) = file_dialog_receiver.try_recv() {
            match dialog_info {
                FileDialogReturnInfo::NewProject(Some(path)) => {
                    new_project_path_buffer = path;
                }
                FileDialogReturnInfo::OpenProject(Some(path)) => {}
                _ => {}
            }
        }

        ui.window("Inspector")
            .build(|| {
                if ui.input_text("Name", &mut name).build() {
                    // called when the field is edited
                    dbg!(&name);
                }
                if ui.checkbox("Enabled", &mut enabled) {}
                unsafe {
                    //if ui.input_float("X", &mut pos_x).build() {}
                    //if ui.input_float2("Y", &mut pos_y).build() {}
                    imgui::sys::igDragFloat2(pos_label.as_ptr(), &mut pos as *mut ImVec2 as *mut f32,
                        0.1, 0.0, 0.0, std::ptr::null(), 0);
                }
            });
        hierarchy_obj.draw_hierarchy(ui);
        ui.window("World")
            .build(|| {
                ui.text("someday this will work");
            });
        ui.window("Files")
            .build(|| {
                ui.text("files go here");
            });
    });
}

struct FileNameInputFilter;
impl imgui::InputTextCallbackHandler for FileNameInputFilter {
    fn char_filter(&mut self, c: char) -> Option<char> {
        // Characters that are problematic for file names.
        // Could be more restrictive and change this to a whitelist.
        const INVALID_CHARS: [char; 11] = ['/', '\\', '?', '%', '*', '*', ':', '|', '"', '<', '>'];
        if INVALID_CHARS.contains(&c) {
            None
        } else {
            Some(c)
        }
    }
}

enum FileDialogReturnInfo {
    NewProject(Option<String>),
    OpenProject(Option<String>)
}

fn new_project(tx: std::sync::mpsc::Sender<FileDialogReturnInfo>) {
    thread::spawn(move || {
        let path = tinyfiledialogs::select_folder_dialog("New Project", "");
        tx.send(FileDialogReturnInfo::NewProject(path)).unwrap();
    });
}

fn open_project(tx: std::sync::mpsc::Sender<FileDialogReturnInfo>) {
    thread::spawn(move || {
        let path = tinyfiledialogs::select_folder_dialog("Open Project", "");
        tx.send(FileDialogReturnInfo::OpenProject(path)).unwrap();
    });
}
