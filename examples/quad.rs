use std::rc::Rc;

use magma::{
    app::App,
    entity::{Entity, Transform},
    model::{Model, Vertex},
};
use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let event_loop = EventLoop::new();
    let mut app = App::new(&event_loop);

    let quad = Rc::new(Model::new(
        app.device.clone(),
        vec![
            Vertex {
                // Bottom-back-left 0
                position: [-0.5, 0.5, -0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                // Bottom-back-right 1
                position: [0.5, 0.5, -0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                // Bottom-front-right 2
                position: [0.5, 0.5, 0.5],
                color: [0.0, 0.0, 1.0],
            },
            Vertex {
                // Bottom-front-left 3
                position: [-0.5, 0.5, 0.5],
                color: [1.0, 1.0, 0.0],
            },
            Vertex {
                // Top-back-left 4
                position: [-0.5, -0.5, -0.5],
                color: [0.0, 1.0, 1.0],
            },
            Vertex {
                // Top-back-right 5
                position: [0.5, -0.5, -0.5],
                color: [1.0, 0.0, 1.0],
            },
            Vertex {
                // Top-front-right 6
                position: [0.5, -0.5, 0.5],
                color: [0.0, 0.0, 0.0],
            },
            Vertex {
                // Top-front-left 7
                position: [-0.5, -0.5, 0.5],
                color: [1.0, 1.0, 1.0],
            },
        ],
        vec![
            4, 0, 3, 4, 3, 7, // Left face
            5, 2, 1, 5, 6, 2, // Right face
            7, 2, 3, 7, 6, 2, // Front face
            5, 0, 1, 5, 4, 0, // Back face
            4, 6, 7, 4, 5, 6, // Top face
            1, 3, 2, 1, 0, 3, // Bottom face
        ],
    ));

    app.add_entity(Entity::new(
        quad.clone(),
        Transform {
            position: cgmath::Vector3::new(0.0, 0.0, 0.5),
            rotation: cgmath::Vector3::new(0.0, 0.0, 0.0),
            scale: cgmath::Vector3::new(0.5, 0.5, 0.5),
        },
    ));

    app.main_loop(event_loop);

    Ok(())
}
