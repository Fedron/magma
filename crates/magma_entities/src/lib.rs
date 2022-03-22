mod component;
mod entity;
mod world;

pub mod prelude {
    pub use crate::component::{Component, Transform};
    pub use crate::entity::Entity;
    pub use crate::world::World;
}
