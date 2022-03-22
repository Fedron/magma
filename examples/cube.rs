use magma::prelude::*;

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new()
        .without_timestamps()
        .init()
        .unwrap();
    
    let app = App::new();
    app.run();

    Ok(())
}
