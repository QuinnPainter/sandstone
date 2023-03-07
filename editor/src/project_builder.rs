use std::io::Write;
use std::process::Command;
use std::path::Path;
use std::str::FromStr;
use quote::quote;
use crate::project_data::ProjectData;

static ARM9_CARGO: &str = include_str!("runtime_files/arm9-cargo.toml");
static ARM9_CARGO_CONFIG: &str = include_str!("runtime_files/arm9-cargo-config.toml");
static ARM7_CARGO: &str = include_str!("runtime_files/arm7-cargo.toml");
static ARM7_CARGO_CONFIG: &str = include_str!("runtime_files/arm7-cargo-config.toml");
static RUST_TOOLCHAIN: &str = include_str!("runtime_files/rust-toolchain.toml");

pub fn build(project_data: &mut ProjectData) {
    // todo: handle IO errors
    log::info!("Starting build...");
    // Create build folder if it doesn't exist
    let build_path = project_data.get_path().join("build");
    std::fs::create_dir_all(&build_path).unwrap();
    let arm9_path = build_path.join("arm9_runtime");
    let arm7_path = build_path.join("arm7_runtime");

    // todo: this code is a travesty. desperately needs cleanup
    let mut user_script_ids: Vec<(u32, &str)> = Vec::new();
    // unfortunately out-dir doesn't work here, so target-dir has to do.
    let user_code_target_path = build_path.join("user-code");
    let rustdoc_command_output = Command::new("rustup")
        .args(["run", "nightly"])
        .arg("cargo")
        .arg("rustdoc")
        .args(["--target-dir", user_code_target_path.to_str().unwrap()])
        .arg("--")
        .args(["--output-format", "json"])
        .current_dir(project_data.get_path().join("code"))
        .output().unwrap();
    if !rustdoc_command_output.status.success() {
        log::error!("Failed to run Rustdoc on user code:\n{}", String::from_utf8_lossy(&rustdoc_command_output.stderr));
        return;
    }
    // todo: would rather not hardcode this path
    let json_path = user_code_target_path.join("thumbv5te-none-eabi/doc/sandstone_user_code.json");
    let json_data: serde_json::Value = serde_json::from_slice(&std::fs::read(json_path).unwrap()).unwrap();
    // Root of the JSON is the Crate, this accesses the Items list
    // that contains all items in the crate in a flat list.
    for (_, item) in json_data["index"].as_object().unwrap() {
        if item["kind"] == "impl" {
            if item["inner"].get("trait").map_or(false, |tr| tr["name"].as_str().map_or(false, |n| n == "HasTypeId")) {
                let docstring = item["docs"].as_str().unwrap();
                if let Some((_, text_after_key)) = docstring.split_once("{script_type_id=") {
                    let type_id = text_after_key.split('}').take(1).next().unwrap().parse::<u32>().unwrap();
                    let script_name = item["inner"]["for"]["inner"]["name"].as_str().unwrap();
                    user_script_ids.push((type_id, script_name));
                }
            }
        }
    }

    log::info!("Found user scripts: {:?}", user_script_ids);
    let (script_ids, script_names): (Vec<u32>, Vec<&str>) = user_script_ids.into_iter().unzip();
    let script_name_tokens = script_names.into_iter().map(|s| proc_macro2::TokenStream::from_str(s).unwrap());

    let arm9_code = quote! {
        #![no_std]
        #![no_main]
        extern crate alloc;
        use core::num::NonZeroU32;
        use alloc::boxed::Box;
        use sandstone_user_code as user_code;

        fn script_factory(id: NonZeroU32) -> Box<dyn sandstone::Script> {
            match u32::from(id) {
                #(#script_ids => Box::new(user_code::#script_name_tokens::default()),)*
                _ => panic!("Invalid script ID: {}", id)
            }
        }

        #[no_mangle]
        extern "C" fn main() -> ! {
            sandstone::hierarchy::init_script_factory(script_factory);
            sandstone::hierarchy::init_prefab_data(include_bytes!("../../graph_data.bin"));
            sandstone::main_loop();
        }
    };
    create_runtime_crate(true, &arm9_path, &arm9_code.to_string());

    let arm7_code = quote! {
        #![no_std]
        #![no_main]
        extern crate alloc;

        #[no_mangle]
        extern "C" fn main() -> ! {
            sandstone_arm7::main_loop();
        }
    };
    create_runtime_crate(false, &arm7_path, &arm7_code.to_string());

    convert_graphical_assets(&project_data);

    let serialised_graphs = sandstone_common::serialize_prefabs(&sandstone_common::SavedPrefabs(project_data.export_saved_graph()));
    let mut graph_file = std::fs::File::create(build_path.join("graph_data.bin")).unwrap();
    graph_file.write_all(&serialised_graphs).unwrap();

    let rom_path = build_path.join(project_data.name.clone() + ".nds");
    match build_rom(&rom_path, &arm9_path, &arm7_path, false) {
        Ok(_) => log::info!("Successfully built {}", rom_path.to_string_lossy()),
        Err(msg) => log::error!("{msg}")
    }
}

pub fn clean_build(project_data: &mut ProjectData) {
    let build_path = project_data.get_path().join("build");
    match std::fs::remove_dir_all(&build_path) {
        Ok(_) => (),
        // totally fine if the folder was not found, we were trying to delete it anyway!
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
        Err(_) => () // todo: handle other errors
    }
    build(project_data);
}

fn build_rom(rom_path: &Path, arm9_path: &Path, arm7_path: &Path, release: bool) -> Result<(), String> {
    build_runtime_crate(arm9_path, release)?;
    build_runtime_crate(arm7_path, release)?;
    ironds_romtool::build_rom(rom_path, &arm9_path.join("arm9"), &arm7_path.join("arm7"))?;
    /*Command::new("ndstool")
        .args(["-c", rom_path.to_str().unwrap()])
        .args(["-9", arm9_path.to_str().unwrap()])
        .args(["-7", arm7_path.to_str().unwrap()])
        .output().unwrap();*/
    Ok(())
}

fn build_runtime_crate(path: &Path, release: bool) -> Result<(), String> {
    let profile = if release { "release" } else { "dev" };
    // Running Cargo directly doesn't work on windows (it doesn't allow us to select Nightly)
    // so we can just run it through rustup and select nightly while we're at it.
    let cargo_output = Command::new("rustup")
        .args(["run", "nightly"])
        .arg("cargo")
        .arg("build")
        .args(["--profile", profile])
        .args(["-Z", "unstable-options"]) // Needed for --out-dir
        .args(["--out-dir", "."]) // Put output binary at the current path
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

fn convert_graphical_assets(project_data: &ProjectData) {
    let output_gfx_path = project_data.get_path().join("build/gfx");
    std::fs::create_dir_all(&output_gfx_path).unwrap();

    for asset_file_path in project_data.graphical_assets.values() {
        let file_stem = asset_file_path.file_stem().unwrap();
        let output_path_base = output_gfx_path.join(file_stem);
        let output_gfx_path = output_path_base.with_extension("gfx");
        let output_pal_path = output_path_base.with_extension("pal");

        let _ = Command::new("superfamiconv")
            .args(["--mode", "gba"])
            .args(["--tile-width", "8"])
            .args(["--tile-height", "8"])
            .args(["--in-image", asset_file_path.to_str().unwrap()])
            .args(["--out-tiles", output_gfx_path.to_str().unwrap()])
            .args(["--out-palette", output_pal_path.to_str().unwrap()])
            .output().unwrap();
    }
}
