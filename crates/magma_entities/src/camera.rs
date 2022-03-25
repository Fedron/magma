use crate::entity::Entity;

#[derive(Debug, Clone, Copy)]
pub struct Camera {
    projection_matrix: glam::Mat4,
    view_matrix: glam::Mat4,
}

impl Camera {
    pub fn new() -> Camera {
        Camera {
            projection_matrix: glam::Mat4::IDENTITY,
            view_matrix: glam::Mat4::IDENTITY,
        }
    }

    pub fn projection_matrix(&self) -> glam::Mat4 {
        self.projection_matrix
    }

    pub fn view_matrix(&self) -> glam::Mat4 {
        self.view_matrix
    }

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
        self.view_matrix = glam::Mat4::IDENTITY;
    }

    pub fn set_perspective(&mut self, fovy: f32, aspect: f32, near: f32, far: f32) {
        self.projection_matrix = glam::Mat4::perspective_rh(fovy, aspect, near, far);
        self.view_matrix = glam::Mat4::IDENTITY;
    }

    pub fn look_at(&mut self, position: glam::Vec3, target: glam::Vec3) {
        self.view_matrix = glam::Mat4::look_at_rh(position, target, glam::Vec3::Y);
    }
}

impl Entity for Camera {
    fn update(&mut self) {
        todo!()
    }

    fn draw(&mut self) {
        todo!()
    }
}
