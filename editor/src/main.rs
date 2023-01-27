mod ui;

use imgui::sys::ImVec2;
use std::ffi::CString;
use std::fs::File;
use std::io::Write;

fn main() {
    let mut saved_graph = dsengine_common::SavedNodeGraph {nodes: Vec::new()};
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
    let a = dsengine_common::serialize_prefabs(&prefabs);
    {
        let mut prefab_file = File::create("../test/prefab_data.bin").unwrap();
        prefab_file.write_all(&a).unwrap();
    }
    println!("{:?}", a);

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

    ui::mainloop(move |ui| {
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
            }
        }
        ui.main_menu_bar(|| {
            ui.menu("File", || {
                ui.menu_item("New");
                ui.menu_item("Open");
                ui.menu("Open Recent", || {
                    ui.menu_item("Placeholder 1");
                    ui.menu_item("Placeholder 2");
                });
                // this Ctrl-S doesn't actually set up that shortcut, just displays the text
                ui.menu_item_config("Save").shortcut("Ctrl+S").build();
                ui.menu_item("Save As..");
                ui.separator();
                ui.menu_item_config("Quit").shortcut("Alt+F4").build();
            });
            if ui.menu_item("About") {}
        });

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
        ui.window("Hierarchy")
            .build(|| {
                if let Some(_t) = ui.tree_node("obj1") {
                    ui.tree_node("obj3");
                }
                ui.tree_node("obj2");
            });
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
