use crate::component::{CameraController, Transform};

pub trait Entity {
    fn update(&self);
    fn draw(&self);
}

pub struct Camera {
    transform: Transform,
    controller: CameraController,
}

impl Entity for Camera {
    fn update(&self) {
        self.controller.get_inputs();
        self.controller.other_function();
        self.controller.calculate_movement(&self.transform);
    }

    fn draw(&self) {}
}
