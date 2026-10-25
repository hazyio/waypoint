mod utils;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::utils::{cli_args::CliArgs, open};
fn main() {
    init_logging();
    tracing::debug!("Logging initialized");

    let mut cli = CliArgs::parse();
    tracing::debug!(
        config = ?cli.config,
        continue_on_error = cli.continue_on_error,
        "CLI arguments parsed"
    );

    let _ = open::open(&mut cli);
}
pub fn init_logging() {
    let log_dir = dirs::cache_dir()
        .expect("Could not find the platform cache directory")
        .join("waypoint");
    std::fs::create_dir_all(&log_dir).expect("Could not create the waypoint log directory");
    let file_appender = tracing_appender::rolling::daily(&log_dir, "waypoint.log");

    // Debug builds log at debug for your crate; release builds stay at info.
    // RUST_LOG still overrides both.
    let default = if cfg!(debug_assertions) { "info,waypoint=debug" } else { "info" };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(file_appender)
        .with_ansi(false) // no color escape codes in the file
        .with_target(true)
        .with_thread_ids(false)
        .init();

    // Send panics to the log file as well as stderr.
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        tracing::error!(%info, "panic");
        prev(info);
    }));

    // Logged after init so it isn't dropped.
    tracing::debug!(log_dir = %log_dir.display(), "Logging initialized");
}