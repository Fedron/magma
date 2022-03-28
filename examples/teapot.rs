use magma::{app::AppWorld, prelude::*, window::WindowBuilder};
use std::{cell::RefCell, path::Path, rc::Rc};

#[derive(PushConstantData)]
pub struct TeapotPushConstantData {
    _transform: Mat4,
    _normal: Mat4,
}

pub struct Teapot {
    pub transform: Transform,
    pub model: Rc<RefCell<Model<TeapotPushConstantData, OBJVertex>>>,
}

impl Entity for Teapot {
    fn update(&mut self) {
        self.transform.rotation = Vec3::new(
            self.transform.rotation.x,
            self.transform.rotation.y + 0.004,
            self.transform.rotation.z,
        );
    }

    fn draw(&mut self) {}
}

pub struct CameraController {
    pub transform: Transform,
    pub camera: Camera,
    pub teapot: Rc<RefCell<Teapot>>,
}

impl Entity for CameraController {
    fn update(&mut self) {
        self.camera.look_at(self.transform.position, Vec3::ZERO);
    }

    fn draw(&mut self) {
        let teapot = self.teapot.borrow_mut();

        teapot
            .model
            .borrow_mut()
            .set_push_constants(TeapotPushConstantData {
                _transform: self.camera.projection_matrix()
                    * self.camera.view_matrix()
                    * teapot.transform.as_matrix(),
                _normal: Mat4::from_mat3(self.transform.as_normal_matrix()),
            });
    }
}

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let mut app = App::new(
        WindowBuilder::new().title("Teapot").build(),
        [0.01, 0.01, 0.01, 1.0],
    );
    let mut teapot_world = World::new();

    let mut teapot_pipeline = app.create_render_pipeline::<TeapotPushConstantData, OBJVertex>(
        &Path::new("shaders/teapot.vert"),
        &Path::new("shaders/teapot.frag"),
        ShaderStageFlag::VERTEX,
    );
    let teapot = Rc::new(RefCell::new(
        Model::<TeapotPushConstantData, OBJVertex>::new_from_file(
            app.device(),
            &Path::new("models/teapot.obj"),
        ),
    ));
    teapot_pipeline.add_model(teapot.clone());
    let teapot = Rc::new(RefCell::new(Teapot {
        transform: Transform {
            position: Vec3::new(0.0, -1.0, 0.0),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        },
        model: teapot,
    }));

    let mut camera = Camera::new();
    camera.set_perspective(50_f32.to_radians(), app.aspect_ratio(), 0.1, 20.0);
    let camera_controller = Rc::new(RefCell::new(CameraController {
        transform: Transform {
            position: Vec3::new(0.0, 2.5, -10.0),
            rotation: Vec3::ZERO,
            scale: Vec3::ONE,
        },
        camera,
        teapot: teapot.clone(),
    }));
    teapot_world.add_entity(camera_controller.clone());
    teapot_world.add_entity(teapot.clone());

    let teapot_app_world = app.add_world(AppWorld::new(teapot_world, camera));
    app.set_active_world(teapot_app_world);
    app.add_render_pipeline(teapot_app_world, teapot_pipeline);
    app.run();

    Ok(())
}
