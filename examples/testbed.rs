use magma::prelude::Window;

fn main() {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let mut window = Window::new(1280, 720, "Magma");

    while !window.should_close() {
        window.poll_events();
    }
}
