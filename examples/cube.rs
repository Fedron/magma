use std::{path::Path, rc::Rc};

use magma::prelude::*;
use memoffset::offset_of;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SimpleVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

// TODO: Make this using a derive macro
impl Vertex for SimpleVertex {
    fn get_attribute_descriptions() -> Vec<VertexAttributeDescription> {
        vec![
            VertexAttributeDescription {
                binding: 0,
                location: 0,
                format: Format::R32G32B32_SFLOAT,
                offset: offset_of!(SimpleVertex, position) as u32,
            },
            VertexAttributeDescription {
                binding: 0,
                location: 1,
                format: Format::R32G32B32_SFLOAT,
                offset: offset_of!(SimpleVertex, color) as u32,
            },
        ]
    }

    fn get_binding_descriptions() -> Vec<VertexBindingDescription> {
        vec![VertexBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<SimpleVertex>() as u32,
            input_rate: VertexInputRate::VERTEX,
        }]
    }
}

pub struct SimplePushConstantData {}
impl PushConstantData for SimplePushConstantData {
    fn as_bytes(&self) -> &[u8]
    where
        Self: Sized,
    {
        unsafe {
            let size_in_bytes = std::mem::size_of::<Self>();
            let size_in_u8 = size_in_bytes / std::mem::size_of::<u8>();
            std::slice::from_raw_parts(self as *const Self as *const u8, size_in_u8)
        }
    }
}

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
