use crate::{components::Transform, mesh::Mesh, pipeline::Pipeline};
use std::rc::Rc;

pub struct Renderable {
    pub mesh: Rc<Mesh>,
    pub pipeline: Rc<Pipeline>,
    pub transform: Transform,
}
