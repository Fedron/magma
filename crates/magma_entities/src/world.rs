use crate::prelude::Entity;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::atomic::{AtomicU32, Ordering},
};

const WORLD_ID: AtomicU32 = AtomicU32::new(0);

/// Contains a collection of [`Entity`]s and handles updating and drawing them
pub struct World {
    /// A unique id for the [`World`]
    id: u32,
    /// All the [`Entity`]s in the world
    entities: Vec<Rc<RefCell<dyn Entity>>>,
}

impl World {
    /// Creates a new [`World`] with no entities and a unique id
    pub fn new() -> World {
        World {
            id: WORLD_ID.fetch_add(1, Ordering::Relaxed),
            entities: Vec::new(),
        }
    }

    /// Adds a new [`Entity`] to the world
    pub fn add_entity(&mut self, entity: Rc<RefCell<dyn Entity>>) {
        self.entities.push(entity);
    }

    /// Runs [`Entity::update`] on each entity in the world
    pub fn update(&mut self) {
        for entity in self.entities.iter_mut() {
            entity.borrow_mut().update();
        }
    }

    /// Runs [`Entity::draw`] on each entity in the world
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
