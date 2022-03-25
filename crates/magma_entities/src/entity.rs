use downcast_rs::{impl_downcast, Downcast};

pub trait Entity: Downcast {
    fn update(&mut self);
    fn draw(&mut self);
}
impl_downcast!(Entity);
