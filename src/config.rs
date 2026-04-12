use crate::model::Field;
use serde::Deserialize;
use std::{env, fs, path::PathBuf};

const CONFIG_SIZE_LIMIT_BYTES: u64 = 100 * 1024;

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
        apply_env_overrides(&mut config);
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

        if fs::metadata(&path)
            .map(|meta| meta.len() > CONFIG_SIZE_LIMIT_BYTES)
            .unwrap_or(false)
        {
            return (
                Self::default(),
                Some(format!(
                    "config file is too large at {}; using defaults",
                    path.display()
                )),
            );
        }

        match fs::read_to_string(&path) {
            Ok(raw) => match serde_yaml::from_str::<Config>(&raw) {
                Ok(config) => (config, None),
                Err(_) => (
                    Self::default(),
                    Some(format!(
                        "invalid config at {}; using defaults",
                        path.display()
                    )),
                ),
            },
            Err(error) => (
                Self::default(),
                Some(format!(
                    "could not read config at {}: {error}; using defaults",
                    path.display()
                )),
            ),
        }
    }
}

fn apply_env_overrides(config: &mut Config) {
    apply_override("ME_VIEW", &mut config.view, parse_view);
    apply_override("ME_ICONS", &mut config.icons, parse_icon_mode);
    apply_override("ME_COLOR", &mut config.color, parse_color_mode);
    apply_override("ME_PLAIN_MODE", &mut config.plain_mode, parse_plain_mode);
    apply_override("ME_WATCH_INTERVAL", &mut config.watch.interval, |raw| {
        raw.parse().ok()
    });
}

fn apply_override<T>(key: &str, slot: &mut T, parse: impl FnOnce(&str) -> Option<T>) {
    if let Ok(raw) = env::var(key)
        && let Some(value) = parse(&raw)
    {
        *slot = value;
    }
}

fn parse_view(raw: &str) -> Option<View> {
    match raw {
        "compact" => Some(View::Compact),
        "block" => Some(View::Block),
        _ => None,
    }
}

fn parse_icon_mode(raw: &str) -> Option<IconMode> {
    match raw {
        "on" => Some(IconMode::On),
        "off" => Some(IconMode::Off),
        "auto" => Some(IconMode::Auto),
        _ => None,
    }
}

fn parse_color_mode(raw: &str) -> Option<ColorMode> {
    match raw {
        "on" => Some(ColorMode::On),
        "off" => Some(ColorMode::Off),
        "auto" => Some(ColorMode::Auto),
        _ => None,
    }
}

fn parse_plain_mode(raw: &str) -> Option<PlainMode> {
    match raw {
        "user" => Some(PlainMode::User),
        "user_host" => Some(PlainMode::UserHost),
        _ => None,
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
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

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
        assert!(!warning.contains("did not find expected"));
        assert_eq!(config.view, View::Block);
        assert_eq!(config.theme, "dark");
    }

    #[test]
    fn load_falls_back_to_defaults_when_config_is_too_large() {
        let temp = tempdir().unwrap();
        let path = temp.path().join(".config/me/config.yaml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "x".repeat(CONFIG_SIZE_LIMIT_BYTES as usize + 1)).unwrap();

        let (config, warning) = Config::load_from_path(Some(path.clone()));

        let warning = warning.expect("expected oversize config warning");
        assert!(warning.contains("config file is too large"));
        assert!(warning.contains(&path.display().to_string()));
        assert_eq!(config.view, View::Block);
    }

    #[test]
    fn load_defaults_color_to_auto() {
        let temp = tempdir().unwrap();
        let path = temp.path().join(".config/me/config.yaml");

        let (config, warning) = Config::load_from_path(Some(path));

        assert!(warning.is_none());
        assert_eq!(config.color, ColorMode::Auto);
    }

    #[test]
    fn env_overrides_apply_valid_values_only() {
        let _guard = ENV_LOCK.lock().unwrap();
        let mut config = Config::default();
        // SAFETY: these tests serialize environment access with `ENV_LOCK`, so
        // mutating process-global environment variables here does not race.
        unsafe {
            env::set_var("ME_VIEW", "compact");
            env::set_var("ME_COLOR", "off");
            env::set_var("ME_WATCH_INTERVAL", "5");
            env::set_var("ME_PLAIN_MODE", "user");
            env::set_var("ME_ICONS", "invalid");
        }

        apply_env_overrides(&mut config);

        // SAFETY: same reasoning as above; the test holds the environment lock.
        unsafe {
            env::remove_var("ME_VIEW");
            env::remove_var("ME_COLOR");
            env::remove_var("ME_WATCH_INTERVAL");
            env::remove_var("ME_PLAIN_MODE");
            env::remove_var("ME_ICONS");
        }

        assert_eq!(config.view, View::Compact);
        assert_eq!(config.color, ColorMode::Off);
        assert_eq!(config.watch.interval, 5);
        assert_eq!(config.plain_mode, PlainMode::User);
        assert_eq!(config.icons, IconMode::Auto);
    }
}
