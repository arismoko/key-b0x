use crate::bindings::ResolvedBindings;
use crate::config::load_or_create;
use crate::platform::{active_keyboard_backend, active_transport};
use crate::runtime_controller::RuntimeWorkerContext;
use crate::state::{RuntimeState, RuntimeStatus, read_runtime_state, write_runtime_state};
use crate::transport::SnapshotEmitter;
use anyhow::{Context, Result, bail};
use key_b0x_core::{InputEvent, MeleeEngine};
use key_b0x_platform::{KeyboardBackend, KeyboardCaptureSession, TransportStatus};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

pub fn run_worker(context: RuntimeWorkerContext) {
    if let Err(error) = run_capture_loop(&context) {
        if !context.cancel.load(Ordering::Relaxed) {
            write_runtime_state(
                &context.state,
                RuntimeState::error(error.to_string()),
                context.listener.as_ref(),
            );
        }
    }
}

fn run_capture_loop(context: &RuntimeWorkerContext) -> Result<()> {
    let config = load_or_create(&context.config_path)
        .with_context(|| format!("failed to load {}", context.config_path.display()))?;

    if config.port != 1 {
        bail!("only port 1 is supported in this proof of concept");
    }

    let backend = active_keyboard_backend();
    let keyboards = backend.list_keyboards()?;
    if keyboards.is_empty() {
        bail!("no keyboards detected");
    }

    let bindings = ResolvedBindings::new(&config)?;
    let mut capture = backend.open()?;
    let mut emitter = SnapshotEmitter::new(active_transport(&config.slippi_user_path, 1)?);
    let mut engine = MeleeEngine::try_new(config.melee.clone()).context("invalid melee config")?;

    apply_transport_status(&context, emitter.emit(&engine.snapshot())?);

    while !context.cancel.load(Ordering::Relaxed) {
        let changes = capture.poll_events()?;
        if changes.is_empty() {
            thread::sleep(Duration::from_millis(4));
            continue;
        }

        for change in changes {
            if let Some(binding) = bindings.lookup(change.key) {
                let snapshot = engine.handle_event(InputEvent {
                    binding,
                    pressed: change.pressed,
                });
                apply_transport_status(&context, emitter.emit(&snapshot)?);
            }

            if context.cancel.load(Ordering::Relaxed) {
                break;
            }
        }
    }

    let neutral = engine.reset();
    if let Ok(status) = emitter.emit(&neutral) {
        apply_transport_status(&context, status);
    }
    capture.release()?;
    Ok(())
}

fn apply_transport_status(context: &RuntimeWorkerContext, status: TransportStatus) {
    let current = read_runtime_state(&context.state);
    if current.status == RuntimeStatus::Stopping {
        return;
    }

    let next_status = match status {
        TransportStatus::WaitingForReader => RuntimeStatus::WaitingForSlippi,
        TransportStatus::Connected | TransportStatus::NewlyConnected => RuntimeStatus::Running,
    };

    if current.status == next_status {
        return;
    }

    write_runtime_state(
        &context.state,
        RuntimeState {
            status: next_status,
            started_at: current.started_at,
            last_error: None,
        },
        context.listener.as_ref(),
    );
}
