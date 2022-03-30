use magma::prelude::{Engine, Window};

fn main() {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let window = Window::new(1280, 720, "Magma");
    let mut engine = Engine::new(window);

    engine.run();
}
