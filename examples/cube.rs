use magma::prelude::*;
use std::{path::Path, rc::Rc};

#[repr(C)]
#[derive(Debug, Clone, Copy, Vertex)]
pub struct SimpleVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

#[derive(PushConstantData)]
pub struct SimplePushConstantData {}

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let mut app = App::new();
    let cube_world = Rc::new(World::new());

    let mut simple_pipeline = app.create_render_pipeline::<SimplePushConstantData, SimpleVertex>(
        &Path::new("simple.vert"),
        &Path::new("simple.frag"),
    );
    simple_pipeline.create_model(
        vec![
            SimpleVertex {
                // Bottom-back-left 0
                position: [-0.5, 0.5, -0.5],
                color: [1.0, 0.0, 0.0],
            },
            SimpleVertex {
                // Bottom-back-right 1
                position: [0.5, 0.5, -0.5],
                color: [0.0, 1.0, 0.0],
            },
            SimpleVertex {
                // Bottom-front-right 2
                position: [0.5, 0.5, 0.5],
                color: [0.0, 0.0, 1.0],
            },
            SimpleVertex {
                // Bottom-front-left 3
                position: [-0.5, 0.5, 0.5],
                color: [1.0, 1.0, 0.0],
            },
            SimpleVertex {
                // Top-back-left 4
                position: [-0.5, -0.5, -0.5],
                color: [0.0, 1.0, 1.0],
            },
            SimpleVertex {
                // Top-back-right 5
                position: [0.5, -0.5, -0.5],
                color: [1.0, 0.0, 1.0],
            },
            SimpleVertex {
                // Top-front-right 6
                position: [0.5, -0.5, 0.5],
                color: [0.0, 0.0, 0.0],
            },
            SimpleVertex {
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
    );

    app.add_world(cube_world.clone());
    app.set_active_world(cube_world.clone());
    app.add_render_pipeline(cube_world.clone(), simple_pipeline);
    app.run();

    Ok(())
}
