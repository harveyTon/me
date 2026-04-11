use crate::model::Field;
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub view: View,
    pub theme: String,
    pub color: ColorMode,
    pub icons: IconMode,
    pub fields: Vec<Field>,
    pub context: ContextConfig,
    pub watch: WatchConfig,
    pub plain_mode: PlainMode,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum View {
    Block,
    Compact,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IconMode {
    Auto,
    On,
    Off,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ColorMode {
    Auto,
    On,
    Off,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlainMode {
    User,
    UserHost,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ContextConfig {
    pub enabled: bool,
    pub project: bool,
    pub container: bool,
    pub ssh: bool,
    pub git: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WatchConfig {
    pub interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            view: View::Block,
            theme: "dark".into(),
            color: ColorMode::Auto,
            icons: IconMode::Auto,
            fields: Vec::new(),
            context: ContextConfig::default(),
            watch: WatchConfig::default(),
            plain_mode: PlainMode::UserHost,
        }
    }
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            project: true,
            container: true,
            ssh: true,
            git: true,
        }
    }
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self { interval: 1 }
    }
}

impl Config {
    pub fn load() -> (Self, Option<String>) {
        let (mut config, warning) = Self::load_from_path(config_path());

        if let Ok(view) = env::var("ME_VIEW") {
            config.view = match view.as_str() {
                "compact" => View::Compact,
                "block" => View::Block,
                _ => config.view,
            };
        }
        if let Ok(icons) = env::var("ME_ICONS") {
            config.icons = match icons.as_str() {
                "on" => IconMode::On,
                "off" => IconMode::Off,
                "auto" => IconMode::Auto,
                _ => config.icons,
            };
        }
        if let Ok(color) = env::var("ME_COLOR") {
            config.color = match color.as_str() {
                "on" => ColorMode::On,
                "off" => ColorMode::Off,
                "auto" => ColorMode::Auto,
                _ => config.color,
            };
        }
        if let Ok(mode) = env::var("ME_PLAIN_MODE") {
            config.plain_mode = match mode.as_str() {
                "user" => PlainMode::User,
                "user_host" => PlainMode::UserHost,
                _ => config.plain_mode,
            };
        }
        if let Some(interval) = env::var("ME_WATCH_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
        {
            config.watch.interval = interval;
        }
        (config, warning)
    }

    pub fn is_light_theme(&self) -> bool {
        self.theme.eq_ignore_ascii_case("light")
    }

    fn load_from_path(path: Option<PathBuf>) -> (Self, Option<String>) {
        let Some(path) = path else {
            return (Self::default(), None);
        };

        if let Err(error) = ensure_config_exists(&path) {
            return (
                Self::default(),
                Some(format!(
                    "warning: could not initialize config at {}: {error}; using defaults",
                    path.display()
                )),
            );
        }

        match fs::read_to_string(&path) {
            Ok(raw) => match serde_yaml::from_str::<Config>(&raw) {
                Ok(config) => (config, None),
                Err(error) => (
                    Self::default(),
                    Some(format!(
                        "warning: invalid config at {}: {error}; fix the file or remove it, using defaults",
                        path.display()
                    )),
                ),
            },
            Err(error) => (
                Self::default(),
                Some(format!(
                    "warning: could not read config at {}: {error}; using defaults",
                    path.display()
                )),
            ),
        }
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".config/me/config.yaml"))
}

fn ensure_config_exists(path: &PathBuf) -> anyhow::Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, default_config_yaml())?;
    Ok(())
}

fn default_config_yaml() -> &'static str {
    "\
view: block
theme: dark
color: auto
icons: auto
fields: []
context:
  enabled: true
  project: true
  container: true
  ssh: true
  git: true
watch:
  interval: 1
plain_mode: user_host
"
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_creates_default_config_when_missing() {
        let temp = tempdir().unwrap();
        let path = temp.path().join(".config/me/config.yaml");

        let (config, warning) = Config::load_from_path(Some(path.clone()));

        assert!(warning.is_none());
        assert!(path.exists());
        let written = fs::read_to_string(path).unwrap();
        assert!(written.contains("view: block"));
        assert!(written.contains("theme: dark"));
        assert!(written.contains("color: auto"));
        assert_eq!(config.theme, "dark");
        assert_eq!(config.view, View::Block);
    }

    #[test]
    fn load_reads_existing_config_values() {
        let temp = tempdir().unwrap();
        let path = temp.path().join(".config/me/config.yaml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            &path,
            "view: compact\ntheme: light\ncolor: off\nplain_mode: user\nwatch:\n  interval: 5\n",
        )
        .unwrap();

        let (config, warning) = Config::load_from_path(Some(path));

        assert!(warning.is_none());
        assert_eq!(config.view, View::Compact);
        assert_eq!(config.theme, "light");
        assert_eq!(config.color, ColorMode::Off);
        assert_eq!(config.plain_mode, PlainMode::User);
        assert_eq!(config.watch.interval, 5);
    }

    #[test]
    fn load_falls_back_to_defaults_when_config_is_invalid() {
        let temp = tempdir().unwrap();
        let path = temp.path().join(".config/me/config.yaml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "view: [broken").unwrap();

        let (config, warning) = Config::load_from_path(Some(path.clone()));

        let warning = warning.expect("expected invalid config warning");
        assert!(warning.contains("invalid config"));
        assert!(warning.contains(&path.display().to_string()));
        assert_eq!(config.view, View::Block);
        assert_eq!(config.theme, "dark");
    }

    #[test]
    fn load_defaults_color_to_auto() {
        let temp = tempdir().unwrap();
        let path = temp.path().join(".config/me/config.yaml");

        let (config, warning) = Config::load_from_path(Some(path));

        assert!(warning.is_none());
        assert_eq!(config.color, ColorMode::Auto);
    }
}
