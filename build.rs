fn main() {
    println!("cargo:rerun-if-changed=shaders/simple-shader/src/lib.rs");

    // Build all the shaders in 'shaders' before-hand so the application doesn't need to compile them at runtime
    let mut shaders_dir = std::env::current_dir().expect("Failed to get current dir");
    shaders_dir.push("shaders");
    let entries = std::fs::read_dir(shaders_dir).expect("Failed to get entries in directory");

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                println!("Compiling {:?} shader crate", path);
                spirv_builder::SpirvBuilder::new(
                    format!("shaders/{}", path.file_name().unwrap().to_str().unwrap()),
                    "spirv-unknown-vulkan1.1",
                )
                .print_metadata(spirv_builder::MetadataPrintout::None)
                .build()
                .expect(&format!(
                    "Failed to build shader {:?}",
                    path.file_name().unwrap()
                ));
            }
        }
    }
}
