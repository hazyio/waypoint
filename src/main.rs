mod utils;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use crate::utils::{cli_args::CliArgs, open};
fn main() {
    init_logging();
    let mut cli = CliArgs::parse();

    let _ = open::open(&mut cli);
}

pub fn init_logging() {
    let log_dir = dirs::cache_dir()
        .expect("Could not find the platform cache directory")
        .join("waypoint");
    std::fs::create_dir_all(&log_dir).expect("Could not create the waypoint log directory");
    let file_appender = tracing_appender::rolling::daily(log_dir, "waypoint.log");

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(file_appender)
        .with_target(true)
        .with_thread_ids(false)
        .init();
}
