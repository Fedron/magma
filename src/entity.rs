use std::{
    rc::Rc,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::model::Model;

static ENTITY_COUNT: AtomicU32 = AtomicU32::new(0);

pub struct Transform {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Vector3<f32>,
    pub scale: cgmath::Vector3<f32>,
}

impl Transform {
    pub fn as_matrix(&self) -> cgmath::Matrix4<f32> {
        let c3 = self.rotation.z.to_radians().cos();
        let s3 = self.rotation.z.to_radians().sin();
        let c2 = self.rotation.x.to_radians().cos();
        let s2 = self.rotation.x.to_radians().sin();
        let c1 = self.rotation.y.to_radians().cos();
        let s1 = self.rotation.y.to_radians().sin();

        cgmath::Matrix4::from_cols(
            cgmath::Vector4::new(
                self.scale.x * (c1 * c3 + s1 * s2 * s3),
                self.scale.x * (c2 * s3),
                self.scale.x * (c1 * s2 * s3 - c3 * s1),
                0.0,
            ),
            cgmath::Vector4::new(
                self.scale.y * (c3 * s1 * s2 - c1 * s3),
                self.scale.y * (c2 * c3),
                self.scale.y * (c1 * c3 * s2 + s1 * s3),
                0.0,
            ),
            cgmath::Vector4::new(
                self.scale.z * (c2 * s1),
                self.scale.z * (-s2),
                self.scale.z * (c1 * c2),
                0.0,
            ),
            cgmath::Vector4::new(self.position.x, self.position.y, self.position.z, 1.0),
        )
    }
}

pub struct Entity {
    _id: u32,
    model: Rc<Model>,
    pub transform: Transform,
}

impl Entity {
    pub fn new(model: Rc<Model>, transform: Transform) -> Entity {
        Entity {
            _id: ENTITY_COUNT.fetch_add(1, Ordering::Relaxed),
            model,
            transform,
        }
    }

    pub fn model(&self) -> &Model {
        self.model.as_ref()
    }

    pub fn transform_matrix(&self) -> cgmath::Matrix4<f32> {
        self.transform.as_matrix()
    }
}
