use magma::prelude::*;

fn main() {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let _instance = Instance::new();
}
