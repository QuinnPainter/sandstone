mod ui;

use dsengine_common::SavedNode;
use imgui::{Ui, sys::ImVec2};
use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::num::NonZeroU32;

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
    println!("{:?}", a);

    let mut hierarchy_graph: Vec<dsengine_common::SavedNode> = Vec::new();
    hierarchy_graph.push(dsengine_common::SavedNode {
        child_index: Some(NonZeroU32::new(1).unwrap()),
        sibling_index: None,
        name: String::from("__EDITOR_ROOT__"),
        transform: dsengine_common::SavedTransform { x: 0, y: 0 },
        script_type_id: None,
        enabled: true
    });
    hierarchy_graph.push(dsengine_common::SavedNode {
        child_index: None,
        sibling_index: Some(NonZeroU32::new(2).unwrap()),
        name: String::from("derp"),
        transform: dsengine_common::SavedTransform { x: 0, y: 0 },
        script_type_id: Some(core::num::NonZeroU32::new(1).unwrap()),
        enabled: true
    });
    hierarchy_graph.push(dsengine_common::SavedNode {
        child_index: Some(NonZeroU32::new(3).unwrap()),
        sibling_index: None,
        name: String::from("herp"),
        transform: dsengine_common::SavedTransform { x: 0, y: 0 },
        script_type_id: Some(core::num::NonZeroU32::new(1).unwrap()),
        enabled: true
    });
    hierarchy_graph.push(dsengine_common::SavedNode {
        child_index: None,
        sibling_index: Some(NonZeroU32::new(4).unwrap()),
        name: String::from("flerp"),
        transform: dsengine_common::SavedTransform { x: 0, y: 0 },
        script_type_id: Some(core::num::NonZeroU32::new(1).unwrap()),
        enabled: true
    });
    hierarchy_graph.push(dsengine_common::SavedNode {
        child_index: None,
        sibling_index: None,
        name: String::from("merp"),
        transform: dsengine_common::SavedTransform { x: 0, y: 0 },
        script_type_id: Some(core::num::NonZeroU32::new(1).unwrap()),
        enabled: true
    });

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
    let mut selected_node_idx: Option<usize> = None;

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
                if ui.is_window_hovered() && ui.is_mouse_clicked(imgui::MouseButton::Right) {
                    ui.open_popup("hierarchy_context");
                }
                if let Some(_p) = ui.begin_popup("hierarchy_context") {
                    if ui.selectable("Add Stuff") {}
                    if ui.selectable("Things") {}
                }
                draw_hierarchy_node(ui, &hierarchy_graph, 0, &mut selected_node_idx);
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

fn draw_hierarchy_node(ui: &Ui, hierarchy: &Vec<dsengine_common::SavedNode>, node_idx: usize, selected_node_idx: &mut Option<usize>) {
    if let Some(node) = hierarchy.get(node_idx) {
        fn draw_child_nodes (ui: &Ui, hierarchy: &Vec<dsengine_common::SavedNode>, selected_node_idx: &mut Option<usize>, node: &SavedNode) {
            if let Some(mut cur_child_idx) = node.child_index {
                loop {
                    let cur_child_idx_usize = u32::from(cur_child_idx) as usize;
                    draw_hierarchy_node(ui, hierarchy, cur_child_idx_usize, selected_node_idx);
                    cur_child_idx = match hierarchy[cur_child_idx_usize].sibling_index {
                        Some(x) => x,
                        None => break
                    }
                }
            }
        }

        // Root node is not drawn
        if node_idx == 0 {
            draw_child_nodes(ui, hierarchy, selected_node_idx, node);
        } else {
            let mut flags = imgui::TreeNodeFlags::empty();
            flags |= imgui::TreeNodeFlags::OPEN_ON_ARROW;
            if node.child_index.is_none() {
                flags |= imgui::TreeNodeFlags::LEAF;
            }
            // could change this to is_some_and if that gets stablised
            // if selected_node_idx.is_some_and(|x| x == node_idx) {
            if matches!(selected_node_idx, Some(x) if *x == node_idx) {
                flags |= imgui::TreeNodeFlags::SELECTED;
            }
            ui.tree_node_config(node.name.as_str()).flags(flags).build(|| {
                if ui.is_item_clicked() {
                    *selected_node_idx = Some(node_idx);
                    println!("clonk: {}", node_idx);
                }
                draw_child_nodes(ui, hierarchy, selected_node_idx, node);
            });
        }
    }
}
