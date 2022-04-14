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

pub struct EmptyPushConstant {}

impl UniformBuffer for EmptyPushConstant {
    fn as_bytes(&self) -> &[u8] {
        &[0]
    }

    fn stage() -> ShaderStageFlags {
        ShaderStageFlags::empty()
    }

    fn get_field_descriptions() -> Vec<UboFieldDescription> {
        vec![]
    }
}

