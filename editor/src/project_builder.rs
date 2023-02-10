use std::io::Write;
use quote::quote;
use crate::project_data::ProjectData;

pub fn build(project_data: &mut ProjectData) {
    // todo: handle IO errors
    // Create build folder if it doesn't exist
    let build_path = project_data.path.join("build");
    std::fs::create_dir_all(&build_path).unwrap();

    let s = quote! {
        #![no_std]
        #![no_main]
        extern crate alloc;

        fn script_factory(id: NonZeroU32) -> Box<dyn Script> {
            match u32::from(id) {
                1 => Box::new(Obj1::default()),
                2 => Box::new(Obj2::default()),
                _ => panic!("Invalid component ID: {}", id)
            }
        }
    };
    // cargo new,
    // .cargo/config.toml (different arm9 arm7)
    // rust-toolchain.toml
    // cargo add user code and dsengine / dsengine_arm7 as dependency
    println!("{}", s.to_string());
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
    let serialised_graphs = dsengine_common::serialize_prefabs(&dsengine_common::SavedPrefabs(project_data.export_saved_graph()));
    let mut graph_file = std::fs::File::create(build_path.join("graph_data.bin")).unwrap();
    graph_file.write_all(&serialised_graphs).unwrap();
}
