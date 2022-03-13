fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let _shader = spirv_builder::SpirvBuilder::new("shaders/simple-shader", "spirv-unknown-vulkan1.1")
        .print_metadata(spirv_builder::MetadataPrintout::None)
        .build()?;
    
    magma::hello();

    Ok(())
}
