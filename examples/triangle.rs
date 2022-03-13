use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let event_loop = EventLoop::new();
    let _window = magma::app::App::init_window(&event_loop);
    magma::app::App::main_loop(event_loop);

    Ok(())
}
