#[derive(Debug)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Vec3,
    pub scale: glam::Vec3,
}

impl Transform {
    pub fn new() -> Transform {
        Transform {
            position: glam::Vec3::ZERO,
            rotation: glam::Vec3::ONE,
            scale: glam::Vec3::ONE,
        }
    }

    pub fn as_matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale,
            glam::Quat::from_euler(
                glam::EulerRot::YXZ,
                self.rotation.y,
                self.rotation.x,
                self.rotation.z,
            ),
            self.position,
        )
    }
}
