pub trait Component {}

impl Component for Transform {}
pub struct Transform {
    position: [f32; 3],
}

impl Component for CameraController {}
pub struct CameraController {
    move_speed: f32,
    look_speed: f32,
}

impl CameraController {
    pub fn get_inputs(&self) {}
    pub fn calculate_movement(&self, transform: &Transform) {}
    pub fn other_function(&self) {}
}
