use std::{cell::RefCell, rc::Rc};

use magma::prelude::*;

pub struct Cube {
    pub inputs: Rc<RefCell<InputHandler>>,
    pub transform: Transform,
    pub color: &'static str,
}

impl Entity for Cube {
    fn update(&mut self) {
        if self.inputs.borrow().is_key_pressed(KeyCode::Up) {
            self.transform.position[1] += 0.1;
        }

        if self.inputs.borrow().is_key_pressed(KeyCode::Down) {
            self.transform.position[1] -= 0.1;
        }

        if self.inputs.borrow().is_key_pressed(KeyCode::Right) {
            self.transform.position[0] += 0.1;
        }

        if self.inputs.borrow().is_key_pressed(KeyCode::Left) {
            self.transform.position[0] -= 0.1;
        }

        println!("I've moved to {:?}", self.transform);

        if self.inputs.borrow().is_key_pressed(KeyCode::R) {
            self.color = "RED";
        }

        if self.inputs.borrow().is_key_pressed(KeyCode::G) {
            self.color = "GREEN";
        }

        if self.inputs.borrow().is_key_pressed(KeyCode::B) {
            self.color = "BLUE";
        }
    }

    fn draw(&mut self) {
        println!("I'm going to be drawn in {}", self.color);
    }
}

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let mut app = App::new();

    let mut cube_world = World::new();
    cube_world.add_entity(Box::new(Cube {
        inputs: app.input_handler.clone(),
        transform: Transform {
            position: [0.0, 0.0, 0.0],
        },
        color: "RED",
    }));
    app.push_world(cube_world);

    app.run();

    Ok(())
}
