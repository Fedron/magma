extern crate log;

mod constants;
mod debug;
mod platforms;
mod utils;
mod window;

pub mod prelude {
    pub use crate::window::Window;
}
