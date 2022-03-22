use crate::prelude::Entity;

pub struct World {
    entities: Vec<Box<dyn Entity>>,
}

impl World {
    pub fn new() -> World {
        World {
            entities: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Box<dyn Entity>) {
        self.entities.push(entity);
    }

    pub fn update(&mut self) {
        for entity in self.entities.iter_mut() {
            entity.update();
        }
    }

    pub fn draw(&mut self) {
        for entity in self.entities.iter_mut() {
            entity.draw();
        }
    }
}
