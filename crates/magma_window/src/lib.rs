//! This crate wraps [`winit`] to make it easier to create windows

mod window;
mod events;

pub mod prelude {
    pub use crate::window::{Window, WindowBuilder};
}
