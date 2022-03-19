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
            projection_matrix: cgmath::Matrix4::new(
                2.0 / (right - left),
                0.0,
                0.0,
                0.0,
                0.0,
                2.0 / (bottom - top),
                0.0,
                0.0,
                0.0,
                0.0,
                1.0 / (far - near),
                0.0,
                -(right + left) / (right - left),
                -(bottom + top) / (bottom - top),
                -near / (far - near),
                1.0
            ),
        }
    }

    pub fn from_perspective(fovy: cgmath::Rad<f32>, aspect: f32, near: f32, far: f32) -> Camera {
        let half_fovy = (fovy.0 / 2.0).tan();
        Camera {
            projection_matrix: cgmath::Matrix4::new(
                1.0 / (aspect * half_fovy),
                0.0,
                0.0,
                0.0,
                0.0,
                1.0 / (half_fovy),
                0.0,
                0.0,
                0.0,
                0.0,
                far / (far - near),
                1.0,
                0.0,
                0.0,
                -(far * near) / (far - near),
                0.0,
            ),
        }
    }
}
