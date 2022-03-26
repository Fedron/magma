//! Bundles all the `magma` crates into one crate that re-exports all the necessary functionality

pub mod app {
    pub use magma_app::prelude::*;
}

pub mod window {
    pub use magma_window::prelude::*;
}

pub mod input {
    pub use magma_input::prelude::*;
}

pub mod entities {
    pub use magma_entities::prelude::*;
}

pub mod render {
    pub use magma_render::prelude::*;
}

pub mod derive {
    pub use magma_derive::{PushConstantData, Vertex};
}

pub mod prelude {
    pub use crate::app::App;
    pub use crate::derive::*;
    pub use crate::entities::*;
    pub use crate::input::*;
    pub use crate::render::*;
    pub use memoffset::offset_of;
}
