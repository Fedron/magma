use cgmath::prelude::*;

pub struct Camera {
    projection_matrix: cgmath::Matrix4<f32>,
    view_matrix: cgmath::Matrix4<f32>,
}

impl Camera {
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
                1.0,
            ),
            view_matrix: cgmath::Matrix4::new(
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
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
            view_matrix: cgmath::Matrix4::new(
                1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
            ),
        }
    }

    pub fn projection_matrix(&self) -> cgmath::Matrix4<f32> {
        self.projection_matrix
    }

    pub fn view_matrix(&self) -> cgmath::Matrix4<f32> {
        self.view_matrix
    }

    pub fn set_view_direction(
        &mut self,
        position: cgmath::Vector3<f32>,
        direction: cgmath::Vector3<f32>,
        up: cgmath::Vector3<f32>,
    ) {
        let w = direction.normalize();
        let u = w.cross(up).normalize();
        let v = w.cross(u);

        self.view_matrix = cgmath::Matrix4::new(
            u.x,
            u.y,
            u.z,
            0.0,
            v.x,
            v.y,
            v.z,
            0.0,
            w.x,
            w.y,
            w.z,
            0.0,
            -u.dot(position),
            -v.dot(position),
            -w.dot(position),
            1.0,
        );
    }

    pub fn set_view_rotation(
        &mut self,
        position: cgmath::Vector3<f32>,
        rotation: cgmath::Vector3<f32>,
    ) {
        let c3 = rotation.z.to_radians().cos();
        let s3 = rotation.z.to_radians().sin();
        let c2 = rotation.x.to_radians().cos();
        let s2 = rotation.x.to_radians().sin();
        let c1 = rotation.y.to_radians().cos();
        let s1 = rotation.y.to_radians().sin();

        let u = cgmath::Vector3::new(c1 * c3 + s1 * s2 * s3, c2 * s3, c1 * s2 * s3 - c3 * s1);
        let v = cgmath::Vector3::new(c3 * s1 * s2 - c1 * s3, c2 * c3, c1 * c3 * s2 + s1 * s3);
        let w = cgmath::Vector3::new(c2 * s1, -s2, c1 * c2);

        self.view_matrix = cgmath::Matrix4::new(
            u.x,
            u.y,
            u.z,
            0.0,
            v.x,
            v.y,
            v.z,
            0.0,
            w.x,
            w.y,
            w.z,
            0.0,
            -u.dot(position),
            -v.dot(position),
            -w.dot(position),
            1.0,
        );
    }
}
