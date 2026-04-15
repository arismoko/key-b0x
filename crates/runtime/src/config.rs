use anyhow::{Context, Result, anyhow, bail};
use key_b0x_core::BindingId;
use key_b0x_platform::NormalizedKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_VERSION: u8 = 2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_version")]
    pub version: u8,
    #[serde(default = "default_slippi_user_path")]
    pub slippi_user_path: PathBuf,
    #[serde(default = "default_port")]
    pub port: u8,
    #[serde(default = "default_bindings")]
    pub bindings: BTreeMap<BindingId, NormalizedKey>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            slippi_user_path: default_slippi_user_path(),
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
                .or_insert_with(|| defaults.get(&binding).copied().unwrap_or(NormalizedKey::KeyA));
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
    let parsed: toml::Value =
        toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))?;
    let version = parsed.get("version").and_then(|value| value.as_integer()).unwrap_or(0);

    if version != i64::from(CONFIG_VERSION) {
        bail!(
            "unsupported key-b0x config version {version} in {}; delete it and rerun `key-b0x-runtime print-default-config` to generate a v{CONFIG_VERSION} config",
            path.display()
        );
    }

    let config: AppConfig =
        parsed.try_into().with_context(|| format!("failed to parse {}", path.display()))?;
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

fn default_port() -> u8 {
    1
}

fn default_slippi_user_path() -> PathBuf {
    super::platform::default_slippi_user_dir()
}

fn default_bindings() -> BTreeMap<BindingId, NormalizedKey> {
    BTreeMap::from([
        (BindingId::AnalogUp, NormalizedKey::BracketRight),
        (BindingId::AnalogDown, NormalizedKey::Digit3),
        (BindingId::AnalogLeft, NormalizedKey::Digit2),
        (BindingId::AnalogRight, NormalizedKey::Digit4),
        (BindingId::ModX, NormalizedKey::KeyV),
        (BindingId::ModY, NormalizedKey::KeyB),
        (BindingId::A, NormalizedKey::KeyM),
        (BindingId::B, NormalizedKey::KeyO),
        (BindingId::L, NormalizedKey::KeyQ),
        (BindingId::R, NormalizedKey::Digit9),
        (BindingId::X, NormalizedKey::KeyP),
        (BindingId::Y, NormalizedKey::Digit0),
        (BindingId::Z, NormalizedKey::BracketLeft),
        (BindingId::CUp, NormalizedKey::KeyK),
        (BindingId::CDown, NormalizedKey::Space),
        (BindingId::CLeft, NormalizedKey::KeyN),
        (BindingId::CRight, NormalizedKey::Comma),
        (BindingId::LightShield, NormalizedKey::Minus),
        (BindingId::MidShield, NormalizedKey::Equal),
        (BindingId::Start, NormalizedKey::Digit7),
        (BindingId::DUp, NormalizedKey::ArrowUp),
        (BindingId::DDown, NormalizedKey::ArrowDown),
        (BindingId::DLeft, NormalizedKey::ArrowLeft),
        (BindingId::DRight, NormalizedKey::ArrowRight),
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
        assert_eq!(config.bindings[&BindingId::AnalogUp], NormalizedKey::BracketRight);
    }

    #[test]
    fn load_or_create_writes_default_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");

        let config = load_or_create(&config_path).unwrap();
        assert!(config_path.exists());
        assert_eq!(config.bindings.len(), BindingId::ALL.len());
    }

    #[test]
    fn old_config_versions_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        fs::write(
            &config_path,
            r#"
version = 1
slippi_user_path = "/tmp/SlippiOnline"
port = 1

[bindings]
analog_up = "KEY_RIGHTBRACE"
"#,
        )
        .unwrap();

        let error = load(&config_path).unwrap_err();
        assert!(error.to_string().contains("unsupported key-b0x config version 1"));
    }
}
