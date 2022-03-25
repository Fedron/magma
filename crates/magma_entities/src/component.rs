//! Contains some pre-defined components that [`Entity`]s could use

/// Represents an objects position, rotation, and scale
#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub position: glam::Vec3,
    pub rotation: glam::Vec3,
    pub scale: glam::Vec3,
}

impl Transform {
    /// Creates a new identity [`Transform`].
    ///
    /// - Positioned at the origin
    /// - No rotation
    /// - Scale of 1 on all axes
    pub fn new() -> Transform {
        Transform {
            position: glam::Vec3::ZERO,
            rotation: glam::Vec3::ZERO,
            scale: glam::Vec3::ONE,
        }
    }

    /// Converts the [`Transform`] into an affine transformation matrix
    ///
    /// Rotation is converted into a [quaternion][glam::Quat] using an euler
    /// rotation sequence of YXZ.
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

/// Represents a projection and view matrix
#[derive(Debug, Clone, Copy)]
pub struct Camera {
    projection_matrix: glam::Mat4,
    view_matrix: glam::Mat4,
}

impl Camera {
    /// Creates a new [`Camera`] with an identity projection and view matrix
    pub fn new() -> Camera {
        Camera {
            projection_matrix: glam::Mat4::IDENTITY,
            view_matrix: glam::Mat4::IDENTITY,
        }
    }

    /// Gets the projection matrix of the camera
    pub fn projection_matrix(&self) -> glam::Mat4 {
        self.projection_matrix
    }

    /// Gets the view matrix of the camera
    pub fn view_matrix(&self) -> glam::Mat4 {
        self.view_matrix
    }

    /// Creates a new orthographic projection matrix.
    /// 
    /// Uses a right-handed coordinate system.
    pub fn set_orthographic(
        &mut self,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
    ) {
        self.projection_matrix = glam::Mat4::orthographic_rh(left, right, bottom, top, near, far);
    }

    /// Creates a new perspective projection matrix.
    /// 
    /// Uses a right-handed coordinate system.
    pub fn set_perspective(&mut self, fovy: f32, aspect: f32, near: f32, far: f32) {
        self.projection_matrix = glam::Mat4::perspective_rh(fovy, aspect, near, far);
    }

    /// Creates a new right-handed view matrix.
    pub fn look_at(&mut self, position: glam::Vec3, target: glam::Vec3) {
        self.view_matrix = glam::Mat4::look_at_rh(position, target, glam::Vec3::Y);
    }
}
