use std::rc::Rc;

use magma::{
    app::App,
    entity::{Entity, Transform2D},
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
                position: [-0.5, -0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5],
                color: [0.0, 0.0, 1.0],
            },
            Vertex {
                position: [-0.5, 0.5],
                color: [1.0, 1.0, 1.0],
            },
        ],
        vec![0, 1, 2, 2, 3, 0],
    ));

    app.add_entity(Entity::new(
        quad.clone(),
        Transform2D {
            position: cgmath::Vector2::new(0.25, 0.0),
            rotation: 0.0,
            scale: cgmath::Vector2::new(1.0, 1.0),
        },
    ));

    app.main_loop(event_loop);

    Ok(())
}
