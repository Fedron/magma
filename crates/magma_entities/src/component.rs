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

    /// Converts the `rotation` and `scale` into a normal matrix that
    /// can be used in shaders with lighting requiring non-uniform scaling.
    pub fn as_normal_matrix(&self) -> glam::Mat3 {
        let c3 = self.rotation.z.to_radians().cos();
        let s3 = self.rotation.z.to_radians().sin();
        let c2 = self.rotation.x.to_radians().cos();
        let s2 = self.rotation.x.to_radians().sin();
        let c1 = self.rotation.y.to_radians().cos();
        let s1 = self.rotation.y.to_radians().sin();
        let inverse_scale = 1.0 / self.scale;

        glam::mat3(
            glam::vec3(
                inverse_scale.x * (c1 * c3 + s1 * s2 * s3),
                inverse_scale.x * (c2 * s3),
                inverse_scale.x * (c1 * s2 * s3 - c3 * s1),
            ),
            glam::vec3(
                inverse_scale.y * (c3 * s1 * s2 - c1 * s3),
                inverse_scale.y * (c2 * c3),
                inverse_scale.y * (c1 * c3 * s2 + s1 * s3),
            ),
            glam::vec3(
                inverse_scale.z * (c2 * s1),
                inverse_scale.z * (-s2),
                inverse_scale.z * (c1 * c2),
            ),
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
        self.view_matrix = glam::Mat4::look_at_rh(position, target, glam::vec3(0.0, -1.0, 0.0));
    }
}
