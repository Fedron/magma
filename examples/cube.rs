use magma::app::App;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();

    let app = App::builder().window_title("Cube Example").build();
    app.main_loop();

    Ok(())
}
