use arb_bot::run;
use dotenv::dotenv;
use fern::colors::{Color, ColoredLevelConfig};
use log::info;
use std::sync::mpsc::channel;

#[cfg(all(not(windows), not(target_env = "musl")))]
#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;
#[tokio::main]
async fn main() {
    dotenv().ok();

    // Set up logging to write messages to stdout and a file
    let mut colors = ColoredLevelConfig::new();
    colors.trace = Color::Cyan;
    colors.debug = Color::Magenta;
    colors.info = Color::Green;
    colors.warn = Color::Red;
    colors.error = Color::BrightRed;

    // setup logging both to stdout and file
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("log.txt").unwrap())
        // hide all logs for everything other than bot
        .level(log::LevelFilter::Info)
        .level_for("arb_bot", log::LevelFilter::Info)
        .level_for("ethers", log::LevelFilter::Error)
        .level_for("ethers-core", log::LevelFilter::Error)
        .filter(|metadata| {
            metadata.target().starts_with("arb_bot")
                || metadata.target().starts_with("other_module")
        })
        .apply()
        .unwrap();

    let (exit_sender, exit_receiver) = channel();

    ctrlc::set_handler(move || {
        exit_sender
            .send(())
            .expect("Could not send signal on channel.")
    })
    .expect("Error setting Ctrl-C handler");

    info!("Starting Arbitrage Bot");
    run(exit_receiver).await;
}
