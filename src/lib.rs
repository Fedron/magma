extern crate log;

mod buffer;
mod components;
mod constants;
mod debug;
mod descriptors;
mod device;
mod engine;
mod mesh;
mod pipeline;
mod platforms;
mod renderer;
mod swapchain;
mod utils;
mod window;

pub mod prelude {
    pub use crate::components::Transform;
    pub use crate::engine::Engine;
    pub use crate::mesh::{
        Format, Mesh, OBJVertex, Vertex, VertexAttributeDescription, VertexBindingDescription,
        VertexInputRate,
    };
    pub use crate::pipeline::Shader;
    pub use crate::renderer::{Renderer, RendererBuilder, Shader as RShader};
    pub use crate::window::Window;
}
