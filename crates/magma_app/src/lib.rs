//! This crate provides high level access to all the sub-crates of as once cohesive application

mod app;

pub mod prelude {
    pub use crate::app::{App, AppWorld};
}
