use crate::pipeline::ShaderStageFlags;

pub trait UniformBuffer {
    fn as_bytes(&self) -> &[u8];
    fn stage() -> ShaderStageFlags;
    fn get_field_descriptions() -> Vec<UboFieldDescription>;
}

#[derive(Debug)]
pub struct UboFieldDescription {
    pub size: usize,
}

