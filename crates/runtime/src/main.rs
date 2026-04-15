mod config;
mod platform;
mod profile;
mod transport;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Parser, Subcommand};
use config::{AppConfig, default_config_path, load_or_create, render_default};
use key_b0x_core::{B0xxEngine, BindingId, InputEvent};
use key_b0x_platform::{
    BackendCapabilities, KeyboardBackend, KeyboardCaptureSession, KeyboardId, NormalizedKey,
};
use signal_hook::consts::signal::{SIGINT, SIGTERM};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;
use key_b0x_platform::TransportStatus;
use transport::SnapshotEmitter;

#[derive(Parser)]
#[command(name = "key-b0x-runtime")]
#[command(about = "Cross-platform Slippi keyboard runtime", version)]
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
        keyboard: Option<KeyboardId>,
        #[cfg(target_os = "linux")]
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
            let slippi_user_path = slippi_user_path.unwrap_or_else(platform::default_slippi_user_dir);
            let installed = profile::install_profile(&slippi_user_path)?;
            println!("Installed {}", installed.profile_path.display());
            if let Some(pipes_path) = installed.pipes_path {
                println!("Created {}", pipes_path.display());
            }
            Ok(())
        }
        #[cfg(target_os = "linux")]
        Command::Run {
            keyboard,
            grab,
            slippi_user_path,
        } => run_command(cli.config, keyboard, grab, slippi_user_path),
        #[cfg(target_os = "windows")]
        Command::Run {
            keyboard,
            slippi_user_path,
        } => run_command(cli.config, keyboard, slippi_user_path),
    }
}

fn list_keyboards_command() -> Result<()> {
    let backend = platform::active_keyboard_backend();
    let auto = backend.auto_detect_keyboard()?.map(|keyboard| keyboard.id);
    let keyboards = backend.list_keyboards()?;
    if keyboards.is_empty() {
        bail!("no keyboards detected");
    }

    for keyboard in keyboards {
        let marker = if auto.as_ref() == Some(&keyboard.id) {
            "*"
        } else {
            " "
        };
        println!("{marker} {}  {}", keyboard.id, keyboard.name);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn run_command(
    config_override: Option<PathBuf>,
    keyboard_override: Option<KeyboardId>,
    grab_override: bool,
    slippi_user_override: Option<PathBuf>,
) -> Result<()> {
    run_command_inner(
        config_override,
        keyboard_override,
        grab_override,
        slippi_user_override,
    )
}

#[cfg(target_os = "windows")]
fn run_command(
    config_override: Option<PathBuf>,
    keyboard_override: Option<KeyboardId>,
    slippi_user_override: Option<PathBuf>,
) -> Result<()> {
    run_command_inner(config_override, keyboard_override, false, slippi_user_override)
}

fn run_command_inner(
    config_override: Option<PathBuf>,
    keyboard_override: Option<KeyboardId>,
    grab_override: bool,
    slippi_user_override: Option<PathBuf>,
) -> Result<()> {
    let mut debug = DebugLogger::from_env()?;
    let config_path = config_override.unwrap_or(default_config_path()?);
    let mut config = load_or_create(&config_path)
        .with_context(|| format!("failed to load {}", config_path.display()))?;
    debug.log(format!("config_path={}", config_path.display()));

    if let Some(slippi_user_path) = slippi_user_override {
        config.slippi_user_path = slippi_user_path;
    }
    if config.port != 1 {
        bail!("only port 1 is supported in this proof of concept");
    }

    let backend = platform::active_keyboard_backend();
    let keyboard_id = resolve_keyboard_id(keyboard_override, &config, &backend)?;
    let exclusive_capture = effective_exclusive_capture(grab_override, &config, backend.capabilities());
    let bindings = ResolvedBindings::new(&config)?;
    debug.log(format!("keyboard_id={keyboard_id}"));
    debug.log(format!(
        "slippi_user_path={}",
        config.slippi_user_path.display()
    ));

    let mut capture = backend.open(&keyboard_id, exclusive_capture)?;
    let mut emitter = SnapshotEmitter::new(platform::active_transport(&config.slippi_user_path, 1)?);
    let stop = Arc::new(AtomicBool::new(false));
    register_signals(&stop)?;

    println!("Using keyboard: {}", capture.info().id);
    println!("Keyboard name: {}", capture.info().name);
    println!("Slippi user path: {}", config.slippi_user_path.display());
    #[cfg(target_os = "linux")]
    println!(
        "Pipe path: {}",
        config
            .slippi_user_path
            .join("Pipes")
            .join("slippibot1")
            .display()
    );
    #[cfg(target_os = "windows")]
    println!("Pipe name: \\\\.\\pipe\\slippibot1");
    if exclusive_capture {
        println!("Exclusive capture enabled");
    }
    debug.log(format!("capture_name={}", capture.info().name));

    let mut engine = B0xxEngine::new();
    let startup_status = emitter.emit(&engine.snapshot())?;
    log_transport_status(&mut debug, "startup_emit", startup_status);

    while !stop.load(Ordering::Relaxed) {
        let changes = capture.poll_events()?;
        if changes.is_empty() {
            thread::sleep(Duration::from_millis(4));
            continue;
        }

        for change in changes {
            debug.log(format!("key_change={} pressed={}", change.key, change.pressed));
            if let Some(binding) = bindings.lookup(change.key) {
                let snapshot = engine.handle_event(InputEvent {
                    binding,
                    pressed: change.pressed,
                });
                debug.log(format!("binding={} pressed={}", binding.label(), change.pressed));
                let status = emitter.emit(&snapshot)?;
                log_transport_status(&mut debug, "emit", status);
            } else {
                debug.log(format!("unbound_key={}", change.key));
            }
        }
    }

    let neutral = engine.reset();
    if let Ok(status) = emitter.emit(&neutral) {
        log_transport_status(&mut debug, "shutdown_emit", status);
    }
    capture.release()?;
    Ok(())
}

fn log_transport_status(debug: &mut DebugLogger, stage: &str, status: TransportStatus) {
    let label = match status {
        TransportStatus::Connected => "connected",
        TransportStatus::NewlyConnected => "newly_connected",
        TransportStatus::WaitingForReader => "waiting_for_reader",
    };
    debug.log(format!("{stage}={label}"));
}

fn register_signals(stop: &Arc<AtomicBool>) -> Result<()> {
    signal_hook::flag::register(SIGINT, Arc::clone(stop)).context("failed to register SIGINT")?;
    signal_hook::flag::register(SIGTERM, Arc::clone(stop)).context("failed to register SIGTERM")?;
    Ok(())
}

fn resolve_keyboard_id<B: KeyboardBackend>(
    override_id: Option<KeyboardId>,
    config: &AppConfig,
    backend: &B,
) -> Result<KeyboardId> {
    if let Some(id) = override_id {
        return Ok(id);
    }
    if let Some(id) = config.keyboard_device.clone() {
        return Ok(id);
    }
    if let Some(keyboard) = backend.auto_detect_keyboard()? {
        return Ok(keyboard.id);
    }
    Err(anyhow!(
        "no keyboard selected and auto-detection did not find a suitable device"
    ))
}

fn effective_exclusive_capture(
    grab_override: bool,
    config: &AppConfig,
    capabilities: BackendCapabilities,
) -> bool {
    capabilities.exclusive_capture && (grab_override || config.exclusive_capture)
}

struct DebugLogger {
    file: Option<std::fs::File>,
}

impl DebugLogger {
    fn from_env() -> Result<Self> {
        let Some(path) = std::env::var_os("KEY_B0X_DEBUG_LOG") else {
            return Ok(Self { file: None });
        };
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .with_context(|| format!("failed to open debug log {}", PathBuf::from(path).display()))?;
        Ok(Self { file: Some(file) })
    }

    fn log(&mut self, message: impl AsRef<str>) {
        if let Some(file) = self.file.as_mut() {
            let _ = writeln!(file, "{}", message.as_ref());
            let _ = file.flush();
        }
    }
}

struct ResolvedBindings {
    bindings: HashMap<NormalizedKey, BindingId>,
}

impl ResolvedBindings {
    fn new(config: &AppConfig) -> Result<Self> {
        let mut bindings = HashMap::new();

        for binding in BindingId::ALL {
            let Some(key) = config.bindings.get(&binding) else {
                bail!("missing binding for {}", binding.label());
            };
            if let Some(existing) = bindings.insert(*key, binding) {
                bail!(
                    "duplicate key assignment: {} is assigned to {} and {}",
                    key,
                    existing.label(),
                    binding.label()
                );
            }
        }

        Ok(Self { bindings })
    }

    fn lookup(&self, key: NormalizedKey) -> Option<BindingId> {
        self.bindings.get(&key).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use key_b0x_platform::{BackendCapabilities, KeyChange, KeyboardInfo};

    struct FakeBackend;

    struct FakeCapture;

    impl KeyboardCaptureSession for FakeCapture {
        fn info(&self) -> &KeyboardInfo {
            panic!("unused")
        }

        fn poll_events(&mut self) -> Result<Vec<KeyChange>> {
            Ok(Vec::new())
        }

        fn release(&mut self) -> Result<()> {
            Ok(())
        }
    }

    impl KeyboardBackend for FakeBackend {
        type Session = FakeCapture;

        fn list_keyboards(&self) -> Result<Vec<KeyboardInfo>> {
            Ok(Vec::new())
        }

        fn auto_detect_keyboard(&self) -> Result<Option<KeyboardInfo>> {
            Ok(None)
        }

        fn open(&self, _id: &KeyboardId, _exclusive: bool) -> Result<Self::Session> {
            Ok(FakeCapture)
        }

        fn capabilities(&self) -> BackendCapabilities {
            BackendCapabilities::default()
        }
    }

    #[test]
    fn resolved_bindings_reject_duplicates() {
        let mut config = AppConfig::default();
        config.bindings.insert(BindingId::A, NormalizedKey::KeyM);
        config.bindings.insert(BindingId::B, NormalizedKey::KeyM);

        assert!(ResolvedBindings::new(&config).is_err());
    }

    #[test]
    fn resolve_keyboard_prefers_override() {
        let config = AppConfig::default();
        let resolved = resolve_keyboard_id(Some(KeyboardId::new("keyboard-99")), &config, &FakeBackend);
        assert_eq!(resolved.unwrap().as_str(), "keyboard-99");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn default_slippi_path_uses_config_dir() {
        assert!(platform::default_slippi_user_dir().ends_with(std::path::Path::new("SlippiOnline")));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn default_slippi_path_uses_slippi_launcher_user_dir() {
        assert!(platform::default_slippi_user_dir().ends_with(
            std::path::Path::new("Slippi Launcher")
                .join("netplay")
                .join("User")
        ));
    }
}
