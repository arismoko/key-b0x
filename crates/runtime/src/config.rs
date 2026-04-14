use anyhow::{Context, Result, anyhow};
use key_b0x_core::BindingId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_VERSION: u8 = 1;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_version")]
    pub version: u8,
    #[serde(default = "default_slippi_user_path")]
    pub slippi_user_path: PathBuf,
    #[serde(default)]
    pub keyboard_device: Option<PathBuf>,
    #[serde(default = "default_exclusive_capture")]
    pub exclusive_capture: bool,
    #[serde(default = "default_port")]
    pub port: u8,
    #[serde(default = "default_bindings")]
    pub bindings: BTreeMap<BindingId, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            slippi_user_path: default_slippi_user_path(),
            keyboard_device: None,
            exclusive_capture: default_exclusive_capture(),
            port: default_port(),
            bindings: default_bindings(),
        }
    }
}

impl AppConfig {
    pub fn normalize(mut self) -> Self {
        let defaults = default_bindings();
        for binding in BindingId::ALL {
            self.bindings
                .entry(binding)
                .or_insert_with(|| defaults.get(&binding).cloned().unwrap_or_default());
        }

        if self.port == 0 {
            self.port = default_port();
        }
        if self.version == 0 {
            self.version = default_version();
        }

        self
    }
}

pub fn default_config_path() -> Result<PathBuf> {
    let config_root = dirs::config_dir().ok_or_else(|| anyhow!("failed to resolve config dir"))?;
    Ok(config_root.join("key-b0x").join("config.toml"))
}

pub fn load_or_create(path: &Path) -> Result<AppConfig> {
    if path.exists() {
        return load(path);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let config = AppConfig::default();
    save(path, &config)?;
    Ok(config)
}

pub fn load(path: &Path) -> Result<AppConfig> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read config {}", path.display()))?;
    let config: AppConfig =
        toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(config.normalize())
}

pub fn save(path: &Path, config: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let raw = render(config)?;
    fs::write(path, raw).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn render_default() -> Result<String> {
    render(&AppConfig::default())
}

fn render(config: &AppConfig) -> Result<String> {
    toml::to_string_pretty(config).context("failed to render config")
}

fn default_version() -> u8 {
    CONFIG_VERSION
}

fn default_exclusive_capture() -> bool {
    false
}

fn default_port() -> u8 {
    1
}

fn default_slippi_user_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("SlippiOnline")
}

fn default_bindings() -> BTreeMap<BindingId, String> {
    BTreeMap::from([
        (BindingId::AnalogUp, "KEY_RIGHTBRACE".to_string()),
        (BindingId::AnalogDown, "KEY_3".to_string()),
        (BindingId::AnalogLeft, "KEY_2".to_string()),
        (BindingId::AnalogRight, "KEY_4".to_string()),
        (BindingId::ModX, "KEY_V".to_string()),
        (BindingId::ModY, "KEY_B".to_string()),
        (BindingId::A, "KEY_M".to_string()),
        (BindingId::B, "KEY_O".to_string()),
        (BindingId::L, "KEY_Q".to_string()),
        (BindingId::R, "KEY_9".to_string()),
        (BindingId::X, "KEY_P".to_string()),
        (BindingId::Y, "KEY_0".to_string()),
        (BindingId::Z, "KEY_LEFTBRACE".to_string()),
        (BindingId::CUp, "KEY_K".to_string()),
        (BindingId::CDown, "KEY_SPACE".to_string()),
        (BindingId::CLeft, "KEY_N".to_string()),
        (BindingId::CRight, "KEY_COMMA".to_string()),
        (BindingId::LightShield, "KEY_MINUS".to_string()),
        (BindingId::MidShield, "KEY_EQUAL".to_string()),
        (BindingId::Start, "KEY_7".to_string()),
        (BindingId::DUp, "KEY_UP".to_string()),
        (BindingId::DDown, "KEY_DOWN".to_string()),
        (BindingId::DLeft, "KEY_LEFT".to_string()),
        (BindingId::DRight, "KEY_RIGHT".to_string()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_contains_all_bindings() {
        let config = AppConfig::default();
        assert_eq!(config.bindings.len(), BindingId::ALL.len());
        assert_eq!(config.port, 1);
        assert!(!config.exclusive_capture);
    }

    #[test]
    fn load_or_create_writes_default_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");

        let config = load_or_create(&config_path).unwrap();
        assert!(config_path.exists());
        assert_eq!(config.bindings.len(), BindingId::ALL.len());
    }
}
