//! This crate provides a system similar to that of Unity's GameObjects for managing entities and worlds

mod component;
mod entity;
mod world;

pub mod prelude {
    pub use crate::component::{Camera, Transform};
    pub use crate::entity::Entity;
    pub use crate::world::World;
    pub use glam::*;
}
