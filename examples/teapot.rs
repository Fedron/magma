use glam::{Mat4, Vec4};
use magma::prelude::*;
use magma_derive::UniformBuffer;

#[derive(UniformBuffer)]
#[ubo(stage = "vertex")]
pub struct SimplePushConstant {
    _model_matrix: Mat4,
    _normal_matrix: Mat4,
}

#[derive(UniformBuffer)]
#[ubo(stage = "both")]
pub struct SimpleUbo {
    _projection: Mat4,
    _view: Mat4,

    _ambient_light: Vec4,
    _light_position: Vec4,
    _light_color: Vec4,
}

fn main() {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let window = Window::new(1280, 720, "Magma");
    let engine = Engine::new(window, [0.01, 0.01, 0.01, 1.0]);

    let _renderer = Renderer::<OBJVertex, SimplePushConstant>::builder(
        engine.device(),
        engine.swapchain_renderpass(),
    )
    .add_shader(Shader {
        file: "shaders/simple.vert",
        entry_point: "main\0",
        stage: Shader::VERTEX,
    })
    .add_shader(Shader {
        file: "shaders/simple.frag",
        entry_point: "main\0",
        stage: Shader::FRAGMENT,
    })
    .add_ubo::<SimpleUbo>(0, 0)
    .build();
}
