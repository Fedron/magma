mod component;
mod entity;
mod world;

pub mod prelude {
    pub use crate::component::Transform;
    pub use crate::entity::Entity;
    pub use crate::world::World;
}
