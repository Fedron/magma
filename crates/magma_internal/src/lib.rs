pub mod app {
    pub use magma_app::prelude::*;
}

pub mod window {
    pub use magma_window::prelude::*;
}

pub mod input {
    pub use magma_input::prelude::*;
}

pub mod prelude {
    pub use crate::app::App;
}
