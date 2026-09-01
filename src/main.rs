mod app;
mod core;
mod fs;
mod plugin;
mod ui;

use app::app::App;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    log::info!("Starting Zero Explorer...");

    let mut app = App::new();
    app.run()?;

    Ok(())
}
