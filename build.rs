fn main() {
    println!("cargo:rerun-if-changed=shaders/shader.vert");
    println!("cargo:rerun-if-changed=shaders/shader.frag");

    // Builds all the GLSL shaders to spirv
    let mut shaders_dir = std::env::current_dir().expect("Failed to get current dir");
    shaders_dir.push("shaders");
    let entries =
        std::fs::read_dir(shaders_dir.clone()).expect("Failed to get entries in shaders directory");

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                // We only want to compile .vert and .frag (not interested in .vert.spv)
                if let Some(shader_extension) = path.extension() {
                    if shader_extension == "spv" {
                        continue;
                    }

                    // println!("Compiling {:?}", path);
                    let mut output_path = path.clone();
                    output_path.set_extension(format!(
                        "{}.spv",
                        path.extension().unwrap().to_str().unwrap()
                    ));
                    let status = std::process::Command::new("glslc")
                        .args(&[path.to_str().unwrap(), "-o", output_path.to_str().unwrap()])
                        .status()
                        .expect("Failed to execute glslc");
                    assert!(status.success(), "Failed to compile {:?}", path);
                }
            }
        }
    }
}
