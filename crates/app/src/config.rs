use anyhow::{Context, Result, anyhow, bail};
use key_b0x_core::{BindingId, MeleeConfig};
use key_b0x_platform::NormalizedKey;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const CONFIG_VERSION: u8 = 2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_version")]
    pub version: u8,
    #[serde(default = "default_slippi_user_path")]
    pub slippi_user_path: PathBuf,
    #[serde(default)]
    pub onboarding_completed: bool,
    #[serde(default = "default_port")]
    pub port: u8,
    #[serde(default = "default_bindings")]
    pub bindings: BTreeMap<BindingId, NormalizedKey>,
    #[serde(default)]
    pub melee: MeleeConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: default_version(),
            slippi_user_path: default_slippi_user_path(),
            onboarding_completed: false,
            port: default_port(),
            bindings: default_bindings(),
            melee: MeleeConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn normalize(mut self) -> Self {
        let defaults = default_bindings();
        for binding in BindingId::ALL {
            self.bindings.entry(binding).or_insert_with(|| {
                defaults
                    .get(&binding)
                    .copied()
                    .unwrap_or(NormalizedKey::KeyA)
            });
        }

        self.port = default_port();
        if self.version == 0 {
            self.version = default_version();
        }
        if self.slippi_user_path.as_os_str().is_empty() {
            self.slippi_user_path = default_slippi_user_path();
        }

        self
    }
}

#[derive(Deserialize)]
struct LegacyConfigV1 {
    #[serde(default)]
    slippi_user_path: Option<PathBuf>,
    #[serde(default)]
    port: Option<u8>,
    #[serde(default)]
    bindings: BTreeMap<BindingId, LegacyBindingKey>,
}

#[derive(Clone, Deserialize)]
#[serde(untagged)]
enum LegacyBindingKey {
    Modern(NormalizedKey),
    Legacy(String),
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
    let version = parsed
        .get("version")
        .and_then(|value| value.as_integer())
        .unwrap_or(1);

    let config = match version {
        1 => {
            let migrated = migrate_v1_config(parsed)
                .with_context(|| format!("failed to migrate {}", path.display()))?;
            save(path, &migrated)?;
            migrated
        }
        2 => parsed
            .try_into()
            .with_context(|| format!("failed to parse {}", path.display()))?,
        _ => {
            bail!(
                "unsupported key-b0x config version {version} in {}; expected v{CONFIG_VERSION}",
                path.display()
            )
        }
    };

    prepare(config).with_context(|| format!("invalid config in {}", path.display()))
}

pub fn save(path: &Path, config: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let normalized = prepare(config.clone())?;
    let raw = toml::to_string_pretty(&normalized).context("failed to render config")?;
    fs::write(path, raw).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn prepare(config: AppConfig) -> Result<AppConfig> {
    let config = config.normalize();
    config.melee.validate().context("invalid melee settings")?;
    Ok(config)
}

pub fn render_default_config() -> Result<String> {
    toml::to_string_pretty(&AppConfig::default()).context("failed to render config")
}

fn migrate_v1_config(parsed: toml::Value) -> Result<AppConfig> {
    let legacy: LegacyConfigV1 = parsed.try_into().context("failed to read v1 config")?;
    let mut config = AppConfig {
        version: CONFIG_VERSION,
        slippi_user_path: legacy
            .slippi_user_path
            .unwrap_or_else(default_slippi_user_path),
        onboarding_completed: false,
        port: legacy.port.unwrap_or_else(default_port),
        bindings: default_bindings(),
        melee: MeleeConfig::default(),
    };

    for (binding, key) in legacy.bindings {
        config.bindings.insert(binding, normalize_legacy_key(&key)?);
    }

    prepare(config)
}

fn normalize_legacy_key(key: &LegacyBindingKey) -> Result<NormalizedKey> {
    match key {
        LegacyBindingKey::Modern(key) => Ok(*key),
        LegacyBindingKey::Legacy(raw) => match raw.as_str() {
            "KEY_0" => Ok(NormalizedKey::Digit0),
            "KEY_1" => Ok(NormalizedKey::Digit1),
            "KEY_2" => Ok(NormalizedKey::Digit2),
            "KEY_3" => Ok(NormalizedKey::Digit3),
            "KEY_4" => Ok(NormalizedKey::Digit4),
            "KEY_5" => Ok(NormalizedKey::Digit5),
            "KEY_6" => Ok(NormalizedKey::Digit6),
            "KEY_7" => Ok(NormalizedKey::Digit7),
            "KEY_8" => Ok(NormalizedKey::Digit8),
            "KEY_9" => Ok(NormalizedKey::Digit9),
            "KEY_A" => Ok(NormalizedKey::KeyA),
            "KEY_B" => Ok(NormalizedKey::KeyB),
            "KEY_C" => Ok(NormalizedKey::KeyC),
            "KEY_D" => Ok(NormalizedKey::KeyD),
            "KEY_E" => Ok(NormalizedKey::KeyE),
            "KEY_F" => Ok(NormalizedKey::KeyF),
            "KEY_G" => Ok(NormalizedKey::KeyG),
            "KEY_H" => Ok(NormalizedKey::KeyH),
            "KEY_I" => Ok(NormalizedKey::KeyI),
            "KEY_J" => Ok(NormalizedKey::KeyJ),
            "KEY_K" => Ok(NormalizedKey::KeyK),
            "KEY_L" => Ok(NormalizedKey::KeyL),
            "KEY_M" => Ok(NormalizedKey::KeyM),
            "KEY_N" => Ok(NormalizedKey::KeyN),
            "KEY_O" => Ok(NormalizedKey::KeyO),
            "KEY_P" => Ok(NormalizedKey::KeyP),
            "KEY_Q" => Ok(NormalizedKey::KeyQ),
            "KEY_R" => Ok(NormalizedKey::KeyR),
            "KEY_S" => Ok(NormalizedKey::KeyS),
            "KEY_T" => Ok(NormalizedKey::KeyT),
            "KEY_U" => Ok(NormalizedKey::KeyU),
            "KEY_V" => Ok(NormalizedKey::KeyV),
            "KEY_W" => Ok(NormalizedKey::KeyW),
            "KEY_X" => Ok(NormalizedKey::KeyX),
            "KEY_Y" => Ok(NormalizedKey::KeyY),
            "KEY_Z" => Ok(NormalizedKey::KeyZ),
            "KEY_MINUS" => Ok(NormalizedKey::Minus),
            "KEY_EQUAL" => Ok(NormalizedKey::Equal),
            "KEY_LEFTBRACE" => Ok(NormalizedKey::BracketLeft),
            "KEY_RIGHTBRACE" => Ok(NormalizedKey::BracketRight),
            "KEY_BACKSLASH" => Ok(NormalizedKey::Backslash),
            "KEY_SEMICOLON" => Ok(NormalizedKey::Semicolon),
            "KEY_APOSTROPHE" => Ok(NormalizedKey::Quote),
            "KEY_GRAVE" => Ok(NormalizedKey::Backquote),
            "KEY_COMMA" => Ok(NormalizedKey::Comma),
            "KEY_DOT" => Ok(NormalizedKey::Period),
            "KEY_SLASH" => Ok(NormalizedKey::Slash),
            "KEY_SPACE" => Ok(NormalizedKey::Space),
            "KEY_TAB" => Ok(NormalizedKey::Tab),
            "KEY_ENTER" => Ok(NormalizedKey::Enter),
            "KEY_BACKSPACE" => Ok(NormalizedKey::Backspace),
            "KEY_ESC" => Ok(NormalizedKey::Escape),
            "KEY_CAPSLOCK" => Ok(NormalizedKey::CapsLock),
            "KEY_LEFTSHIFT" => Ok(NormalizedKey::ShiftLeft),
            "KEY_RIGHTSHIFT" => Ok(NormalizedKey::ShiftRight),
            "KEY_LEFTCTRL" => Ok(NormalizedKey::ControlLeft),
            "KEY_RIGHTCTRL" => Ok(NormalizedKey::ControlRight),
            "KEY_LEFTALT" => Ok(NormalizedKey::AltLeft),
            "KEY_RIGHTALT" => Ok(NormalizedKey::AltRight),
            "KEY_LEFTMETA" => Ok(NormalizedKey::MetaLeft),
            "KEY_RIGHTMETA" => Ok(NormalizedKey::MetaRight),
            "KEY_UP" => Ok(NormalizedKey::ArrowUp),
            "KEY_DOWN" => Ok(NormalizedKey::ArrowDown),
            "KEY_LEFT" => Ok(NormalizedKey::ArrowLeft),
            "KEY_RIGHT" => Ok(NormalizedKey::ArrowRight),
            _ => bail!("unknown legacy binding key: {raw}"),
        },
    }
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
        assert_eq!(
            config.bindings[&BindingId::AnalogUp],
            NormalizedKey::BracketRight
        );
        assert_eq!(config.melee, MeleeConfig::default());
        assert!(!config.onboarding_completed);
    }

    #[test]
    fn load_or_create_writes_default_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");

        let config = load_or_create(&config_path).unwrap();
        assert!(config_path.exists());
        assert_eq!(config.bindings.len(), BindingId::ALL.len());
        assert_eq!(config.version, CONFIG_VERSION);
    }

    #[test]
    fn v1_configs_are_migrated_and_rewritten() {
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

        let config = load(&config_path).unwrap();

        assert_eq!(config.version, CONFIG_VERSION);
        assert_eq!(
            config.bindings[&BindingId::AnalogUp],
            NormalizedKey::BracketRight
        );
        let rewritten = fs::read_to_string(config_path).unwrap();
        assert!(rewritten.contains("version = 2"));
        assert!(rewritten.contains("analog_up = \"BracketRight\""));
    }

    #[test]
    fn newer_config_versions_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        fs::write(
            &config_path,
            r#"
version = 9
slippi_user_path = "/tmp/SlippiOnline"
port = 1
"#,
        )
        .unwrap();

        let error = load(&config_path).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("unsupported key-b0x config version 9")
        );
    }

    #[test]
    fn version_two_configs_without_melee_section_load_with_defaults() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        fs::write(
            &config_path,
            r#"
version = 2
slippi_user_path = "/tmp/SlippiOnline"
port = 1

[bindings]
analog_up = "BracketRight"
analog_down = "Digit3"
analog_left = "Digit2"
analog_right = "Digit4"
mod_x = "KeyV"
mod_y = "KeyB"
a = "KeyM"
b = "KeyO"
l = "KeyQ"
r = "Digit9"
x = "KeyP"
y = "Digit0"
z = "BracketLeft"
c_up = "KeyK"
c_down = "Space"
c_left = "KeyN"
c_right = "Comma"
light_shield = "Minus"
mid_shield = "Equal"
start = "Digit7"
d_up = "ArrowUp"
d_down = "ArrowDown"
d_left = "ArrowLeft"
d_right = "ArrowRight"
"#,
        )
        .unwrap();

        let config = load(&config_path).unwrap();
        assert_eq!(config.melee, MeleeConfig::default());
        assert!(!config.onboarding_completed);
    }

    #[test]
    fn save_normalizes_port_and_validates_melee() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("config.toml");
        let mut config = AppConfig::default();
        config.port = 9;

        save(&config_path, &config).unwrap();
        let loaded = load(&config_path).unwrap();

        assert_eq!(loaded.port, 1);
    }
}
