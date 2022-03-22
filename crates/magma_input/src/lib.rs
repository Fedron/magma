mod keyboard;
mod handler;

pub mod prelude {
    pub use crate::keyboard::{KeyCode, KeyState, KeyboardInput};
    pub use crate::handler::InputHandler;
}
