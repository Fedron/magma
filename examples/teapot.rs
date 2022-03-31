use std::{path::Path, rc::Rc};

use glam::{vec3, Vec3};
use magma::prelude::{Engine, Mesh, OBJVertex, Renderable, Shader, Transform, Window};

fn main() {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let window = Window::new(1280, 720, "Magma");
    let mut engine = Engine::new(window, [0.01, 0.01, 0.01, 1.0]);

    let teapot = Mesh::new_from_file(engine.device(), &Path::new("models/teapot.obj"));
    let quad = Rc::new(Mesh::new(
        engine.device(),
        &[
            OBJVertex {
                position: [-0.5, -0.5, 0.0],
                normal: [0.0, 0.0, -1.0],
                color: [1.0, 1.0, 1.0],
            },
            OBJVertex {
                position: [0.5, -0.5, 0.0],
                normal: [0.0, 0.0, -1.0],
                color: [1.0, 1.0, 1.0],
            },
            OBJVertex {
                position: [0.5, 0.5, 0.0],
                normal: [0.0, 0.0, -1.0],
                color: [1.0, 1.0, 1.0],
            },
            OBJVertex {
                position: [-0.5, 0.5, 0.0],
                normal: [0.0, 0.0, -1.0],
                color: [1.0, 1.0, 1.0],
            },
        ],
        &[0, 3, 1, 1, 3, 2],
    ));
    let pipeline = Rc::new(engine.create_pipeline::<OBJVertex>(&[
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
        pipeline: pipeline.clone(),
        transform: Transform {
            position: -Vec3::Y,
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        },
    };
    let floor = Renderable {
        mesh: quad.clone(),
        pipeline: pipeline.clone(),
        transform: Transform {
            position: -Vec3::Y,
            rotation: vec3(90.0, 0.0, 0.0),
            scale: Vec3::ONE * 10.0,
        },
    };
    let wall = Renderable {
        mesh: quad.clone(),
        pipeline: pipeline.clone(),
        transform: Transform {
            position: vec3(0.0, 2.5, 5.0),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE * 10.0,
        },
    };

    engine.add_renderable(teapot);
    engine.add_renderable(floor);
    engine.add_renderable(wall);

    engine.run();
}
