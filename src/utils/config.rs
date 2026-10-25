use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigSetting {
    pub name: String,
    pub cmd: String,
    pub dir: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Config {
    pub settings: Vec<ConfigSetting>,
    #[serde(skip)]
    pub by_command: HashMap<String, Vec<ConfigSetting>>,
}

#[derive(Debug, Clone, Deserialize)]
struct RawConfigSetting {
    pub cmd: String,
    pub dir: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
}

impl Config {
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::debug!(path = %path.display(), "Loading config file");
        let content = std::fs::read_to_string(path)?;
        let raw: HashMap<String, RawConfigSetting> = noyalib::from_str(&content)?;

        let config_base_dir = path
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        tracing::debug!(
            base_dir = %config_base_dir.display(),
            entries = raw.len(),
            "Config parsed"
        );

        let mut settings = Vec::with_capacity(raw.len());
        let mut by_command: HashMap<String, Vec<ConfigSetting>> = HashMap::new();

        for (name, value) in raw {
            let resolved_dir = if value.dir.is_absolute() {
                tracing::debug!(entry = %name, dir = %value.dir.display(), "Using absolute config directory");
                value.dir.clone()
            } else {
                let joined = config_base_dir.join(value.dir);
                tracing::debug!(entry = %name, dir = %joined.display(), "Resolved relative config directory");
                joined
            };

            if !resolved_dir.exists() {
                return Err(Box::new(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "Config entry '{}' points to a missing directory: {}",
                        name,
                        resolved_dir.display()
                    ),
                )));
            }

            if !resolved_dir.is_dir() {
                return Err(Box::new(Error::new(
                    ErrorKind::InvalidInput,
                    format!(
                        "Config entry '{}' dir is not a directory: {}",
                        name,
                        resolved_dir.display()
                    ),
                )));
            }

            let setting = ConfigSetting {
                name,
                cmd: value.cmd,
                dir: resolved_dir,
                args: value.args,
            };

            tracing::debug!(
                entry = %setting.name,
                cmd = %setting.cmd,
                args_count = setting.args.len(),
                dir = %setting.dir.display(),
                "Config entry accepted"
            );

            by_command
                .entry(setting.cmd.clone())
                .or_default()
                .push(setting.clone());

            settings.push(setting);
        }

        tracing::debug!(
            settings_count = settings.len(),
            command_groups = by_command.len(),
            "Config index built"
        );

        Ok(Self {
            settings,
            by_command,
        })
    }

    pub fn by_command(&self, command: &str) -> Option<&Vec<ConfigSetting>> {
        self.by_command.get(command)
    }
}
