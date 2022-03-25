mod camera;
mod component;
mod entity;
mod world;

pub mod prelude {
    pub use crate::camera::Camera;
    pub use crate::component::Transform;
    pub use crate::entity::Entity;
    pub use crate::world::World;
    pub use glam::*;
}
