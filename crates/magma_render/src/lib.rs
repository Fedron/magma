extern crate log;

mod constants;
mod debug;
mod device;
mod model;
mod pipeline;
mod platforms;
mod render_system;
mod renderer;
mod swapchain;
mod utils;

pub mod prelude {
    pub use crate::pipeline::PipelineConfigInfo;
    pub use crate::render_system::*;
    pub use crate::renderer::Renderer;

    #[repr(align(16))]
    #[derive(Debug, Clone, Copy)]
    pub struct Align16<T>(pub T);
}
