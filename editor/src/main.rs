mod ui;

use imgui::Condition;

fn main() {
    let mut first_loop = true;
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


                let hierarchy_name = std::ffi::CString::new("Hierarchy").unwrap();
                let files_name = std::ffi::CString::new("Files").unwrap();
                let inspector_name = std::ffi::CString::new("Inspector").unwrap();
                let world_name = std::ffi::CString::new("World").unwrap();
                imgui::sys::igDockBuilderDockWindow(hierarchy_name.as_ptr(), dock_id_hierarchy);
                imgui::sys::igDockBuilderDockWindow(files_name.as_ptr(), dock_id_files);
                imgui::sys::igDockBuilderDockWindow(inspector_name.as_ptr(), dock_id_inspector);
                imgui::sys::igDockBuilderDockWindow(world_name.as_ptr(), tmp_id);
                imgui::sys::igDockBuilderFinish(dockspace_id);
            }
        }
        ui.main_menu_bar(|| {
            let _ = ui.menu_item("Stuff");
            let _ = ui.menu_item("things");
        });

        ui.window("Inspector")
            .size([100.0, 50.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("Gday\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\neef");
            });
        ui.window("Hierarchy")
            .size([100.0, 50.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("mate");
            });
        ui.window("World")
            .size([100.0, 50.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("hoerl");
            });
        ui.window("Files")
            .size([100.0, 50.0], Condition::FirstUseEver)
            .build(|| {
                ui.text("wefg");
            });
    });
}
