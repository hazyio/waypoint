use rustyline::DefaultEditor;
use std::{path::PathBuf, process::Command};

use crate::utils::cli_args;
use crate::utils::config::Config;

use super::builtin;
use super::prompt::Prompt;

pub fn open(cli_args: &mut cli_args::CliArgs) -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    let prompt = Prompt::new();
    let mut oldpwd: Option<PathBuf> = None;
    tracing::info!(
        "Interactive shell opened, use ctrl+d to exit; use waypoint to confirm if you are in the waypoint environment"
    );
    loop {
        let cwd: PathBuf = std::env::current_dir().unwrap_or_default();
        let Ok(line) = rl.readline(&prompt.render(&cwd)) else {
            break;
        };
        let line = line.trim();
        if let Some(reason) = allow(line) {
            eprintln!("waypoint: blocked ({reason})");
            continue;
        }
        if line.is_empty() {
            continue;
        }
        rl.add_history_entry(line)?;

        let (cmd, arg) = line
            .split_once(char::is_whitespace)
            .map(|(c, a)| (c, a.trim()))
            .unwrap_or((line, ""));

        match cmd {
            "exit" => break,
            "waypoint-update-config" => {
                let new_path = cwd.join("waypoint.yaml");
                if !new_path.exists() {
                    tracing::error!("Config file not found at: {}", new_path.display());
                    continue;
                }
                cli_args.update_config_path(new_path.clone());
                println!("Config path updated to: {}", new_path.display());
                continue;
            }
            "waypoint" => {
                println!("You are in the waypoint environment.");
                continue;
            }
            "cd" => {
                builtin::cd(arg, &mut oldpwd);
                continue;
            }
            _ => {}
        }

        #[cfg(windows)]
        let mut command = Command::new("cmd");
        #[cfg(not(windows))]
        let mut command = Command::new("sh");

        #[cfg(windows)]
        command.arg("/C");
        #[cfg(not(windows))]
        command.arg("-c");
        let read_config = Config::load_from_file(&cli_args.config_path());
        match read_config {
            Ok(value) => {
                if let Some(data) = value.by_command(cmd) {
                    for data in data {
                        if data.args.
                    }
                    // command = command.current_dir(dir)
                    // If the command is found in the config, set the current directory to the one specified in the config
                }
            }
            Err(e) => {
                tracing::error!("Failed to read config file: {}", e);
                if cli_args.continue_on_error {
                    continue;
                }
            }
        }

        let _ = command.arg(line).env("WAYPOINT_SHELL", "1").status();
    }
    Ok(())
}

fn allow(cmd: &str) -> Option<&str> {
    if cmd.contains("rm -rf /") {
        return Some("rm -rf / is blocked");
    }
    None
}
