use ash::vk;

pub type VkFormat = ash::vk::Format;

#[derive(Clone, Copy)]
pub enum VertexInputRate {
    Vertex,
    Instance,
}

impl Into<vk::VertexInputRate> for VertexInputRate {
    fn into(self) -> vk::VertexInputRate {
        match self {
            VertexInputRate::Vertex => vk::VertexInputRate::VERTEX,
            VertexInputRate::Instance => vk::VertexInputRate::INSTANCE,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VertexAttributeDescription {
    pub location: u32,
    pub binding: u32,
    pub format: VkFormat,
    pub offset: u32,
}

impl Into<vk::VertexInputAttributeDescription> for VertexAttributeDescription {
    fn into(self) -> vk::VertexInputAttributeDescription {
        vk::VertexInputAttributeDescription {
            location: self.location,
            binding: self.binding,
            format: self.format,
            offset: self.offset,
        }
    }
}

#[derive(Clone, Copy)]
pub struct VertexBindingDescription {
    pub binding: u32,
    pub stride: u32,
    pub input_rate: VertexInputRate,
}

impl Into<vk::VertexInputBindingDescription> for VertexBindingDescription {
    fn into(self) -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: self.binding,
            stride: self.stride,
            input_rate: self.input_rate.into(),
        }
    }
}

pub trait Vertex {
    fn get_attribute_descriptions() -> Vec<VertexAttributeDescription>;
    fn get_binding_descriptions() -> Vec<VertexBindingDescription>;
}

pub struct EmptyVertex {}

impl Vertex for EmptyVertex {
    fn get_attribute_descriptions() -> Vec<VertexAttributeDescription> {
        vec![]
    }

    fn get_binding_descriptions() -> Vec<VertexBindingDescription> {
        vec![]
    }
}

