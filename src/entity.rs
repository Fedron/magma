use std::{
    rc::Rc,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::model::Model;

static ENTITY_COUNT: AtomicU32 = AtomicU32::new(0);

pub struct Transform2D {
    pub position: cgmath::Vector2<f32>,
    pub rotation: f32,
    pub scale: cgmath::Vector2<f32>,
}

impl Transform2D {
    pub fn as_matrix(&self) -> cgmath::Matrix2<f32> {
        let sin = self.rotation.to_radians().sin();
        let cos = self.rotation.to_radians().cos();

        let rotation_matrix = cgmath::Matrix2::new(cos, sin, -sin, cos);
        let scale_matrix = cgmath::Matrix2::new(self.scale.x, 0.0, 0.0, self.scale.y);

        rotation_matrix * scale_matrix
    }
}

pub struct Entity {
    _id: u32,
    model: Rc<Model>,
    pub transform: Transform2D,
}

impl Entity {
    pub fn new(model: Rc<Model>, transform: Transform2D) -> Entity {
        Entity {
            _id: ENTITY_COUNT.fetch_add(1, Ordering::Relaxed),
            model,
            transform,
        }
    }

    pub fn model(&self) -> &Model {
        self.model.as_ref()
    }

    pub fn transform_matrix(&self) -> cgmath::Matrix2<f32> {
        self.transform.as_matrix()
    }
}
