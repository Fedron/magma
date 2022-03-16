use magma::{
    app::App,
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

    app.add_model(Model::new(
        app.device.clone(),
        vec![
            Vertex {
                position: [-0.25, -0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.25, 0.5],
                color: [1.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.75, 0.5],
                color: [1.0, 0.0, 0.0],
            },
        ],
    ));
    app.add_model(Model::new(
        app.device.clone(),
        vec![
            Vertex {
                position: [0.0, -0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5],
                color: [0.0, 1.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5],
                color: [0.0, 1.0, 0.0],
            },
        ],
    ));
    app.add_model(Model::new(
        app.device.clone(),
        vec![
            Vertex {
                position: [0.25, -0.5],
                color: [0.0, 0.0, 5.0],
            },
            Vertex {
                position: [0.75, 0.5],
                color: [0.0, 0.0, 5.0],
            },
            Vertex {
                position: [-0.25, 0.5],
                color: [0.0, 0.0, 5.0],
            },
        ],
    ));
    
    app.main_loop(event_loop);

    Ok(())
}
