use magma::app::App;
use winit::event_loop::EventLoop;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let event_loop = EventLoop::new();
    let window = App::init_window(&event_loop);
    let app = App::new();
    app.main_loop(event_loop, window);

    Ok(())
}
