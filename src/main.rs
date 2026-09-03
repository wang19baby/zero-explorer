mod app;
mod core;
mod fs;
mod plugin;
mod ui;

use app::App;

fn main() -> anyhow::Result<()> {
    // 初始化文件日志
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}:{}] {}",
                chrono::Local::now().format("%H:%M:%S%.3f"),
                record.level(),
                record.file().unwrap_or("?"),
                record.line().unwrap_or(0),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::Dispatch::new().chain(std::fs::File::create("zero_explorer.log")?))
        .chain(std::io::stdout())
        .apply()?;

    log::info!("=== Zero Explorer Starting ===");

    let mut app = App::new();

    app.run()?;

    Ok(())
}
