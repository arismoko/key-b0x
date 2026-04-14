mod config;
mod profile;
mod transport;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use config::{AppConfig, default_config_path, load_or_create, render_default};
use evdev::KeyCode;
use key_b0x_core::{B0xxEngine, BindingId, InputEvent};
use key_b0x_platform_linux::{
    KeyboardCapture, auto_detect_keyboard, key_code_from_name, list_keyboards,
};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;
use transport::{LinuxTransport, SnapshotEmitter};

#[derive(Parser)]
#[command(name = "key-b0x-runtime")]
#[command(about = "Linux-first Slippi keyboard runtime", version)]
struct Cli {
    #[arg(long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ListKeyboards,
    PrintDefaultConfig,
    InstallProfile {
        #[arg(long)]
        slippi_user_path: Option<PathBuf>,
    },
    Run {
        #[arg(long)]
        keyboard: Option<PathBuf>,
        #[arg(long)]
        grab: bool,
        #[arg(long)]
        slippi_user_path: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::ListKeyboards => list_keyboards_command(),
        Command::PrintDefaultConfig => {
            print!("{}", render_default()?);
            Ok(())
        }
        Command::InstallProfile { slippi_user_path } => {
            let slippi_user_path = slippi_user_path.unwrap_or_else(default_slippi_user_dir);
            let profile_path = profile::install_profile(&slippi_user_path)?;
            println!("Installed {}", profile_path.display());
            println!("Created {}", slippi_user_path.join("Pipes").display());
            Ok(())
        }
        Command::Run {
            keyboard,
            grab,
            slippi_user_path,
        } => run_command(cli.config, keyboard, grab, slippi_user_path),
    }
}

fn list_keyboards_command() -> Result<()> {
    let auto = auto_detect_keyboard().map(|keyboard| keyboard.path);
    let keyboards = list_keyboards();
    if keyboards.is_empty() {
        bail!("no keyboards detected");
    }

    for keyboard in keyboards {
        let marker = if auto.as_ref() == Some(&keyboard.path) {
            "*"
        } else {
            " "
        };
        println!("{marker} {}  {}", keyboard.path.display(), keyboard.name);
    }

    Ok(())
}

fn run_command(
    config_override: Option<PathBuf>,
    keyboard_override: Option<PathBuf>,
    grab_override: bool,
    slippi_user_override: Option<PathBuf>,
) -> Result<()> {
    let config_path = config_override.unwrap_or(default_config_path()?);
    let mut config = load_or_create(&config_path)
        .with_context(|| format!("failed to load {}", config_path.display()))?;

    if let Some(slippi_user_path) = slippi_user_override {
        config.slippi_user_path = slippi_user_path;
    }
    if config.port != 1 {
        bail!("only port 1 is supported in this proof of concept");
    }

    let keyboard_path = resolve_keyboard_path(keyboard_override, &config)?;
    let exclusive_capture = grab_override || config.exclusive_capture;
    let bindings = ResolvedBindings::new(&config)?;

    let mut capture = KeyboardCapture::open(&keyboard_path, exclusive_capture)?;
    let mut emitter = SnapshotEmitter::new(LinuxTransport::new(&config.slippi_user_path, 1)?);
    let stop = Arc::new(AtomicBool::new(false));
    register_signals(&stop)?;

    println!("Using keyboard: {}", capture.info().path.display());
    println!("Keyboard name: {}", capture.info().name);
    println!("Slippi user path: {}", config.slippi_user_path.display());
    println!(
        "Pipe path: {}",
        config
            .slippi_user_path
            .join("Pipes")
            .join("slippibot1")
            .display()
    );
    if exclusive_capture {
        println!("Exclusive capture enabled");
    }

    let mut engine = B0xxEngine::new();
    emitter.emit(&engine.snapshot())?;

    while !stop.load(Ordering::Relaxed) {
        let changes = capture.poll_events()?;
        if changes.is_empty() {
            thread::sleep(Duration::from_millis(4));
            continue;
        }

        for change in changes {
            if let Some(binding) = bindings.lookup(change.code) {
                let snapshot = engine.handle_event(InputEvent {
                    binding,
                    pressed: change.pressed,
                });
                emitter.emit(&snapshot)?;
            }
        }
    }

    let neutral = engine.reset();
    let _ = emitter.emit(&neutral);
    capture.release()?;
    Ok(())
}

fn register_signals(stop: &Arc<AtomicBool>) -> Result<()> {
    signal_hook::flag::register(SIGINT, Arc::clone(stop)).context("failed to register SIGINT")?;
    signal_hook::flag::register(SIGTERM, Arc::clone(stop)).context("failed to register SIGTERM")?;
    Ok(())
}

fn resolve_keyboard_path(override_path: Option<PathBuf>, config: &AppConfig) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return Ok(path);
    }
    if let Some(path) = config.keyboard_device.clone() {
        return Ok(path);
    }
    if let Some(keyboard) = auto_detect_keyboard() {
        return Ok(keyboard.path);
    }
    Err(anyhow!(
        "no keyboard selected and auto-detection did not find a suitable device"
    ))
}

fn default_slippi_user_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("SlippiOnline")
}

struct ResolvedBindings {
    bindings: HashMap<KeyCode, BindingId>,
}

impl ResolvedBindings {
    fn new(config: &AppConfig) -> Result<Self> {
        let mut bindings = HashMap::new();

        for binding in BindingId::ALL {
            let Some(key_name) = config.bindings.get(&binding) else {
                bail!("missing binding for {}", binding.label());
            };
            let key_code = key_code_from_name(key_name)?;
            if let Some(existing) = bindings.insert(key_code, binding) {
                bail!(
                    "duplicate key assignment: {} is assigned to {} and {}",
                    key_name,
                    existing.label(),
                    binding.label()
                );
            }
        }

        Ok(Self { bindings })
    }

    fn lookup(&self, code: KeyCode) -> Option<BindingId> {
        self.bindings.get(&code).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_bindings_reject_duplicates() {
        let mut config = AppConfig::default();
        config.bindings.insert(BindingId::A, "KEY_M".to_string());
        config.bindings.insert(BindingId::B, "KEY_M".to_string());

        assert!(ResolvedBindings::new(&config).is_err());
    }

    #[test]
    fn resolve_keyboard_prefers_override() {
        let config = AppConfig::default();
        let resolved = resolve_keyboard_path(Some(PathBuf::from("/dev/input/event99")), &config);
        assert_eq!(resolved.unwrap(), PathBuf::from("/dev/input/event99"));
    }

    #[test]
    fn default_slippi_path_uses_config_dir() {
        assert!(default_slippi_user_dir().ends_with(std::path::Path::new("SlippiOnline")));
    }
}
