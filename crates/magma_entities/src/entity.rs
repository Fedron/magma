use downcast_rs::{impl_downcast, Downcast};

/// An entity that can be added to a [`World`]
pub trait Entity: Downcast {
    fn update(&mut self);
    fn draw(&mut self);
}
impl_downcast!(Entity);
