use magma::{app::AppWorld, prelude::*, window::WindowBuilder};
use std::{cell::RefCell, path::Path, rc::Rc};

#[repr(C)]
#[derive(Debug, Clone, Copy, Vertex)]
pub struct SimpleVertex {
    #[location = 0]
    pub position: [f32; 3],
    #[location = 1]
    pub color: [f32; 3],
}

#[derive(PushConstantData)]
pub struct SimplePushConstantData {
    _transform: Mat4,
}

pub struct Cube {
    pub transform: Transform,
    pub model: Rc<RefCell<Model<SimplePushConstantData, SimpleVertex>>>,
}

impl Entity for Cube {
    fn update(&mut self) {
        self.transform.rotation = Vec3::new(
            self.transform.rotation.x + 0.002,
            self.transform.rotation.y + 0.004,
            self.transform.rotation.z,
        );
    }

    fn draw(&mut self) {}
}

pub struct CameraController {
    pub transform: Transform,
    pub camera: Camera,
    pub cube: Rc<RefCell<Cube>>,
}

impl Entity for CameraController {
    fn update(&mut self) {
        self.camera.look_at(self.transform.position, Vec3::ZERO);
    }

    fn draw(&mut self) {
        let cube = self.cube.borrow_mut();

        cube.model
            .borrow_mut()
            .set_push_constants(SimplePushConstantData {
                _transform: self.camera.projection_matrix()
                    * self.camera.view_matrix()
                    * cube.transform.as_matrix(),
            });
    }
}

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let mut app = App::new(
        WindowBuilder::new().title("Cube").build(),
        [0.1, 0.1, 0.1, 1.0],
    );
    let mut cube_world = World::new();

    let mut simple_pipeline = app.create_render_pipeline::<SimplePushConstantData, SimpleVertex>(
        &Path::new("shaders/cube.vert"),
        &Path::new("shaders/cube.frag"),
        ShaderStageFlag::VERTEX,
    );
    let cube = simple_pipeline.create_model(
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
    let cube = Rc::new(RefCell::new(Cube {
        transform: Transform::new(),
        model: cube,
    }));

    let mut camera = Camera::new();
    camera.set_perspective(50_f32.to_radians(), app.aspect_ratio(), 0.1, 10.0);
    let camera_controller = Rc::new(RefCell::new(CameraController {
        transform: Transform {
            position: Vec3::new(0.0, 0.0, -3.0),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        },
        camera,
        cube: cube.clone(),
    }));
    cube_world.add_entity(camera_controller.clone());
    cube_world.add_entity(cube.clone());

    let cube_app_world = app.add_world(AppWorld::new(cube_world, camera));
    app.set_active_world(cube_app_world);
    app.add_render_pipeline(cube_app_world, simple_pipeline);
    app.run();

    Ok(())
}
