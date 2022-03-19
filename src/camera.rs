pub struct Camera {
    projection_matrix: cgmath::Matrix4<f32>,
}

impl Camera {
    pub fn projection_matrix(&self) -> cgmath::Matrix4<f32> {
        self.projection_matrix
    }

    pub fn from_orthographic(
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
    ) -> Camera {
        Camera {
            projection_matrix: cgmath::ortho(left, right, bottom, top, near, far),
        }
    }

    pub fn from_perspective(fovy: cgmath::Rad<f32>, aspect: f32, near: f32, far: f32) -> Camera {
        Camera {
            projection_matrix: cgmath::perspective(fovy, aspect, near, far),
        }
    }
}
