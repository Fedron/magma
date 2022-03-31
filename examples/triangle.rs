use magma::prelude::*;
use magma_derive::Vertex;
use memoffset::offset_of;

#[derive(Vertex)]
pub struct TriangleVertex {
    #[location = 0]
    position: [f32; 3],
    #[location = 1]
    color: [f32; 3],
}

fn main() {
    let window = Window::new(1280, 720, "Magma");
    let mut engine = Engine::new(window, [0.01, 0.01, 0.01, 1.0]);

    let mut renderer =
        Renderer::<TriangleVertex>::builder(engine.device(), engine.swapchain_renderpass())
            .add_shader(Shader {
                file: "shaders/triangle.vert",
                entry_point: "main\0",
                stage: Shader::VERTEX,
            })
            .add_shader(Shader {
                file: "shaders/triangle.frag",
                entry_point: "main\0",
                stage: Shader::FRAGMENT,
            })
            .build();
    renderer.add_mesh(Mesh::new(
        engine.device(),
        &[
            TriangleVertex {
                position: [0.0, -0.5, 0.0],
                color: [1.0, 0.0, 0.0],
            },
            TriangleVertex {
                position: [-0.5, 0.5, 0.0],
                color: [0.0, 1.0, 0.0],
            },
            TriangleVertex {
                position: [0.5, 0.5, 0.0],
                color: [0.0, 0.0, 1.0],
            },
        ],
        &[0, 1, 2],
    ));

    engine.add_renderer(renderer);
    engine.run();
}
