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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_test_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock drift")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("waypoint-{label}-{nanos}"));
        fs::create_dir_all(&dir).expect("failed to create temp test dir");
        dir
    }

    #[test]
    fn loads_and_groups_settings_by_command() {
        let root = temp_test_dir("config-load-group");
        let tools_dir = root.join("tools");
        let crates_dir = root.join("crates");
        fs::create_dir_all(&tools_dir).expect("failed to create tools dir");
        fs::create_dir_all(&crates_dir).expect("failed to create crates dir");

        let config_path = root.join("waypoint.yaml");
        let config_content = r#"
bun:
  cmd: bun
  dir: tools
bun-add:
  cmd: bun
  dir: tools
  args:
    - add
cargo:
  cmd: cargo
  dir: crates
"#;
        fs::write(&config_path, config_content).expect("failed to write config");

        let config = Config::load_from_file(&config_path).expect("config should load");

        assert_eq!(config.settings.len(), 3);

        let bun_group = config.by_command("bun").expect("bun group should exist");
        assert_eq!(bun_group.len(), 2);
        assert!(bun_group.iter().any(|s| s.name == "bun"));
        assert!(bun_group.iter().any(|s| s.name == "bun-add"));
        assert!(bun_group.iter().all(|s| s.dir == tools_dir));

        let cargo_group = config
            .by_command("cargo")
            .expect("cargo group should exist");
        assert_eq!(cargo_group.len(), 1);
        assert_eq!(cargo_group[0].dir, crates_dir);

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn fails_when_directory_is_missing() {
        let root = temp_test_dir("config-missing-dir");
        let config_path = root.join("waypoint.yaml");
        let config_content = r#"
bun:
  cmd: bun
  dir: does-not-exist
"#;
        fs::write(&config_path, config_content).expect("failed to write config");

        let err = Config::load_from_file(&config_path).expect_err("expected missing-dir error");
        let message = err.to_string();
        assert!(message.contains("missing directory"));
        assert!(message.contains("bun"));

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn fails_when_dir_is_not_a_directory() {
        let root = temp_test_dir("config-not-directory");
        let file_path = root.join("not-a-dir");
        fs::write(&file_path, "content").expect("failed to create file");

        let config_path = root.join("waypoint.yaml");
        let config_content = r#"
bun:
  cmd: bun
  dir: not-a-dir
"#;
        fs::write(&config_path, config_content).expect("failed to write config");

        let err = Config::load_from_file(&config_path).expect_err("expected not-a-directory error");
        let message = err.to_string();
        assert!(message.contains("is not a directory"));
        assert!(message.contains("bun"));

        let _ = fs::remove_dir_all(&root);
    }
}
