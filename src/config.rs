use crate::model::Field;
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub view: View,
    pub theme: String,
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
        }
    }
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self { interval: 1 }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let mut config = config_path()
            .filter(|path| path.exists())
            .map(|path| {
                let raw = fs::read_to_string(path)?;
                anyhow::Ok(serde_yaml::from_str::<Config>(&raw)?)
            })
            .transpose()?
            .unwrap_or_default();

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
        Ok(config)
    }
}

fn config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".config/me/config.yaml"))
}
