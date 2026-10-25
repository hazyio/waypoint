use clap::Parser;

use std::{env, path::PathBuf, process::exit};
#[derive(Parser)]
#[command(name = "waypoint", version, about)]
pub struct CliArgs {
    /// Path to the config file. Defaults to `waypoint.yaml` in the current directory.
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
    /// Keep going when reading the config file fails.
    #[arg(long, global = true)]
    pub continue_on_error: bool,
}
impl CliArgs {
    pub fn config_path(&self) -> PathBuf {
        match &self.config {
            Some(path) => {
                tracing::debug!(path = %path.display(), "Using explicit config path");
                if !path.exists() {
                    tracing::error!("Config file not found at: {}", path.display());
                    exit(1);
                }
                path.clone()
            }
            None => {
                let path = default_config_path();
                tracing::debug!(path = %path.display(), "Using default config path");
                path
            }
        }
    }
    pub fn update_config_path(&mut self, new_path: PathBuf) {
        tracing::debug!(path = %new_path.display(), "Updating config path");
        self.config = Some(new_path);
    }
}
fn default_config_path() -> PathBuf {
    let path = env::current_dir().expect("Failed to read working directory.");

    path.join("waypoint.yaml")
}
