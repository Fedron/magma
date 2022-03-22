use magma::app::App;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let app = App::new();
    app.main_loop();

    Ok(())
}
