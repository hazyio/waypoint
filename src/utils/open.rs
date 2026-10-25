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
        tracing::debug!(line = line, "Read input line");
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
        tracing::debug!(cmd = cmd, arg = arg, "Parsed command line");

        match cmd {
            "exit" => break,
            "waypoint-update-config" => {
                let new_path = cwd.join("waypoint.yaml");
                if !new_path.exists() {
                    tracing::error!("Config file not found at: {}", new_path.display());
                    continue;
                }
                cli_args.update_config_path(new_path.clone());
                tracing::debug!(path = %new_path.display(), "Config path updated from shell command");
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
        match Config::load_from_file(&cli_args.config_path()) {
            Ok(value) => {
                if let Some(settings) = value.by_command(cmd) {
                    tracing::debug!(
                        cmd = cmd,
                        candidates = settings.len(),
                        "Found command config candidates"
                    );
                    if let Some(setting) = settings
                        .iter()
                        .find(|setting| line_matches_setting_args(arg, &setting.args))
                    {
                        tracing::debug!(
                            setting = %setting.name,
                            dir = %setting.dir.display(),
                            "Matched config setting"
                        );
                        command.current_dir(&setting.dir);
                    } else {
                        tracing::debug!(
                            cmd = cmd,
                            arg = arg,
                            "No config setting matched command args"
                        );
                    }
                } else {
                    tracing::debug!(cmd = cmd, "No config group for command");
                }
            }
            Err(e) => {
                tracing::error!("Failed to read config file: {}", e);
                if cli_args.continue_on_error {
                    continue;
                }
            }
        };

        tracing::debug!(line = line, "Executing shell command");
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

fn line_matches_setting_args(line_args: &str, required_args: &[String]) -> bool {
    let line_parts: Vec<&str> = line_args.split_whitespace().collect();

    // all-or-none: empty config args only match empty input args.
    if required_args.is_empty() {
        return line_parts.is_empty();
    }

    required_args
        .iter()
        .all(|required| line_parts.iter().any(|part| part == required))
}

#[cfg(test)]
mod tests {
    use super::line_matches_setting_args;

    #[test]
    fn empty_required_args_match_only_empty_input() {
        let required: Vec<String> = vec![];
        assert!(line_matches_setting_args("", &required));
        assert!(!line_matches_setting_args("add", &required));
    }

    #[test]
    fn requires_all_config_args_to_be_present() {
        let required = vec!["add".to_string(), "--dev".to_string()];
        assert!(line_matches_setting_args("add --dev", &required));
        assert!(line_matches_setting_args("--dev add", &required));
        assert!(!line_matches_setting_args("add", &required));
        assert!(!line_matches_setting_args("--dev", &required));
    }

    #[test]
    fn allows_extra_user_args_when_all_required_are_present() {
        let required = vec!["add".to_string()];
        assert!(line_matches_setting_args("add lodash", &required));
    }
}
