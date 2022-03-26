//! This crate contains abstraction of the Vulkan API using [`ash`]

extern crate log;

mod constants;
mod debug;
mod device;
mod model;
mod pipeline;
mod platforms;
mod renderer;
mod swapchain;
mod utils;

pub mod prelude {
    pub use crate::model::Model;
    pub use crate::pipeline::{Pipeline, PipelineConfigInfo, RenderPipeline};
    pub use crate::renderer::{
        Format, PushConstantData, Renderer, Vertex, VertexAttributeDescription,
        VertexBindingDescription, VertexInputRate,
    };

    /// Aligns a type by 16 bytes
    #[repr(align(16))]
    #[derive(Debug, Clone, Copy)]
    pub struct Align16<T>(pub T);
}
