extern crate log;

mod buffer;
mod constants;
mod debug;
mod device;
mod engine;
mod mesh;
mod platforms;
mod swapchain;
mod utils;
mod window;

pub mod prelude {
    pub use crate::engine::Engine;
    pub use crate::mesh::{Mesh, Vertex};
    pub use crate::window::Window;
}
