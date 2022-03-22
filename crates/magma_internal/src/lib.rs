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

pub mod prelude {
    pub use crate::app::App;
    pub use crate::entities::*;
    pub use crate::input::*;
}
