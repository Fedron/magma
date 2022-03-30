use std::{path::Path, rc::Rc};

use glam::Vec3;
use magma::prelude::{Engine, Mesh, Renderable, Shader, SimpleVertex, Transform, Window};

fn main() {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let window = Window::new(1280, 720, "Magma");
    let mut engine = Engine::new(window, [0.01, 0.01, 0.01, 1.0]);

    let teapot = Mesh::new_from_file(engine.device(), &Path::new("models/teapot.obj"));
    let pipeline = Rc::new(engine.create_pipeline::<SimpleVertex>(&[
        Shader {
            file: "shaders/simple.vert".to_string(),
            entry_point: "main\0".to_string(),
            stage: Shader::VERTEX,
        },
        Shader {
            file: "shaders/simple.frag".to_string(),
            entry_point: "main\0".to_string(),
            stage: Shader::FRAGMENT,
        },
    ]));

    let teapot = Renderable {
        mesh: Rc::new(teapot),
        pipeline,
        transform: Transform {
            position: -Vec3::Y,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        },
    };
    engine.add_renderable(teapot);

    engine.run();
}
