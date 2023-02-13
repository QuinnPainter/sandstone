use std::io::Write;
use std::process::Command;
use std::path::Path;
use quote::quote;
use crate::project_data::ProjectData;

static ARM9_CARGO: &str = include_str!("runtime_files/arm9-cargo.toml");
static ARM9_CARGO_CONFIG: &str = include_str!("runtime_files/arm9-cargo-config.toml");
static ARM7_CARGO: &str = include_str!("runtime_files/arm7-cargo.toml");
static ARM7_CARGO_CONFIG: &str = include_str!("runtime_files/arm7-cargo-config.toml");
static RUST_TOOLCHAIN: &str = include_str!("runtime_files/rust-toolchain.toml");

pub fn build(project_data: &mut ProjectData) {
    // todo: handle IO errors
    // Create build folder if it doesn't exist
    let build_path = project_data.path.join("build");
    std::fs::create_dir_all(&build_path).unwrap();
    let arm9_path = build_path.join("arm9_runtime");
    let arm7_path = build_path.join("arm7_runtime");

    let arm9_code = quote! {
        #![no_std]
        #![no_main]
        extern crate alloc;
        use core::num::NonZeroU32;
        use alloc::boxed::Box;
        use dsengine_user_code as user_code;

        fn script_factory(id: NonZeroU32) -> Box<dyn dsengine::Script> {
            match u32::from(id) {
                1 => Box::new(user_code::Obj1::default()),
                2 => Box::new(user_code::Obj2::default()),
                _ => panic!("Invalid script ID: {}", id)
            }
        }

        #[no_mangle]
        extern "C" fn main() -> ! {
            dsengine::hierarchy::init_script_factory(script_factory);
            dsengine::hierarchy::init_prefab_data(include_bytes!("../../graph_data.bin"));
            dsengine::main_loop();
        }
    };
    create_runtime_crate(true, &arm9_path, &arm9_code.to_string());

    let arm7_code = quote! {
        #![no_std]
        #![no_main]
        extern crate alloc;

        #[no_mangle]
        extern "C" fn main() -> ! {
            dsengine_arm7::main_loop();
        }
    };
    create_runtime_crate(false, &arm7_path, &arm7_code.to_string());
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

    let rom_path = build_path.join(project_data.name.clone() + ".nds");
    match build_rom(&rom_path, &arm9_path, &arm7_path, false) {
        Ok(_) => (),
        Err(msg) => println!("{msg}")
    }
}

fn build_rom(rom_path: &Path, arm9_path: &Path, arm7_path: &Path, release: bool) -> Result<(), String> {
    build_runtime_crate(arm9_path, release)?;
    build_runtime_crate(arm7_path, release)?;
    ironds_romtool::build_rom(rom_path, arm9_path, arm7_path)?;
    Ok(())
}

fn build_runtime_crate(path: &Path, release: bool) -> Result<(), String> {
    let profile = if release { "release" } else { "dev" };
    let cargo_output = Command::new("cargo")
        .arg("build")
        .args(["--profile", profile])
        .current_dir(path)
        .output().unwrap();
    if cargo_output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&cargo_output.stderr).to_string())
    }
}

fn create_runtime_crate(arm9: bool, path: &Path, code: &str) {
    let (cargo_toml, cargo_config);
    if arm9 {
        cargo_toml = ARM9_CARGO;
        cargo_config = ARM9_CARGO_CONFIG;
    } else {
        cargo_toml = ARM7_CARGO;
        cargo_config = ARM7_CARGO_CONFIG;
    }
    // Regenerate crate if it doesn't exist
    if !path.join("Cargo.toml").exists() {
        // Even if the cargo toml is missing, folder may still exist. in that case, delete it so we get a clean slate
        match std::fs::remove_dir_all(&path) {
            Ok(_) => (),
            // totally fine if the folder was not found, we were trying to delete it anyway!
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
            Err(_) => () // todo: handle other errors
        }

        // Create directories
        std::fs::create_dir_all(path.join(".cargo")).unwrap();
        std::fs::create_dir_all(path.join("src")).unwrap();

        // Create files
        std::fs::write(path.join("Cargo.toml"), cargo_toml).unwrap();
        std::fs::write(path.join(".cargo/config.toml"), cargo_config).unwrap();
        std::fs::write(path.join("rust-toolchain.toml"), RUST_TOOLCHAIN).unwrap();
    }
    std::fs::write(path.join("src/main.rs"), code).unwrap();
}
 