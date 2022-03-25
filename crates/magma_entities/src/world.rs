use crate::prelude::Entity;
use std::{cell::RefCell, rc::Rc};

pub struct World {
    id: u32,
    entities: Vec<Rc<RefCell<dyn Entity>>>,
}

impl World {
    pub fn new() -> World {
        World {
            id: 0,
            entities: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: Rc<RefCell<dyn Entity>>) {
        self.entities.push(entity);
    }

    pub fn update(&mut self) {
        for entity in self.entities.iter_mut() {
            entity.borrow_mut().update();
        }
    }

    pub fn draw(&mut self) {
        for entity in self.entities.iter_mut() {
            entity.borrow_mut().draw();
        }
    }
}

impl std::hash::Hash for World {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for World {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for World {}
