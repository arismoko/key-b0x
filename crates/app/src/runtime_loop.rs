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
use std::time::{Duration, Instant};

const SHUTDOWN_WAIT_INTERVAL: Duration = Duration::from_millis(50);
const RECONNECT_PROBE_INTERVAL: Duration = Duration::from_millis(75);
#[cfg(debug_assertions)]
const DEV_LATENCY_REPORT_INTERVAL: Duration = Duration::from_secs(1);

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
    let mut last_reconnect_probe_at = Instant::now();
    let mut latency_logger = DevLatencyLogger::new();

    let initial_status = emitter.emit(&engine.snapshot())?;
    record_transport_status(context, initial_status, &mut last_reconnect_probe_at);
    let mut last_transport_status = initial_status;

    while !context.cancel.load(Ordering::Relaxed) {
        let changes = capture.wait_for_events(next_wait_timeout(
            last_transport_status,
            last_reconnect_probe_at.elapsed(),
        ))?;
        if changes.is_empty() {
            if should_probe_reconnect(last_transport_status, last_reconnect_probe_at.elapsed()) {
                let status = emitter.emit(&engine.snapshot())?;
                record_transport_status(context, status, &mut last_reconnect_probe_at);
                last_transport_status = status;
            }
            continue;
        }

        for change in changes {
            if let Some(binding) = bindings.lookup(change.key) {
                let snapshot = engine.handle_event(InputEvent {
                    binding,
                    pressed: change.pressed,
                });
                let status = emitter.emit(&snapshot)?;
                record_transport_status(context, status, &mut last_reconnect_probe_at);
                last_transport_status = status;
                latency_logger.record(change.observed_at.elapsed(), status);
            }

            if context.cancel.load(Ordering::Relaxed) {
                break;
            }
        }
    }

    let neutral = engine.reset();
    latency_logger.flush();
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

fn record_transport_status(
    context: &RuntimeWorkerContext,
    status: TransportStatus,
    last_reconnect_probe_at: &mut Instant,
) {
    if status == TransportStatus::WaitingForReader {
        *last_reconnect_probe_at = Instant::now();
    }

    apply_transport_status(context, status);
}

fn should_probe_reconnect(last_status: TransportStatus, elapsed_since_probe: Duration) -> bool {
    last_status == TransportStatus::WaitingForReader
        && elapsed_since_probe >= RECONNECT_PROBE_INTERVAL
}

fn next_wait_timeout(last_status: TransportStatus, elapsed_since_probe: Duration) -> Duration {
    let reconnect_wait = match last_status {
        TransportStatus::WaitingForReader => {
            RECONNECT_PROBE_INTERVAL.saturating_sub(elapsed_since_probe)
        }
        TransportStatus::Connected | TransportStatus::NewlyConnected => SHUTDOWN_WAIT_INTERVAL,
    };

    reconnect_wait.min(SHUTDOWN_WAIT_INTERVAL)
}

#[cfg(debug_assertions)]
struct DevLatencyLogger {
    interval_started_at: Instant,
    sample_count: u32,
    total_latency_ns: u128,
    max_latency: Duration,
    last_latency: Duration,
}

#[cfg(debug_assertions)]
impl DevLatencyLogger {
    fn new() -> Self {
        Self {
            interval_started_at: Instant::now(),
            sample_count: 0,
            total_latency_ns: 0,
            max_latency: Duration::ZERO,
            last_latency: Duration::ZERO,
        }
    }

    fn record(&mut self, latency: Duration, status: TransportStatus) {
        if status == TransportStatus::WaitingForReader {
            return;
        }

        self.sample_count += 1;
        self.total_latency_ns += latency.as_nanos();
        self.max_latency = self.max_latency.max(latency);
        self.last_latency = latency;

        if self.interval_started_at.elapsed() >= DEV_LATENCY_REPORT_INTERVAL {
            self.flush();
        }
    }

    fn flush(&mut self) {
        if self.sample_count == 0 {
            self.interval_started_at = Instant::now();
            return;
        }

        let average_latency_ns = self.total_latency_ns / u128::from(self.sample_count);
        eprintln!(
            "[key-b0x][dev] input->emit latency: last={:.3}ms avg={:.3}ms max={:.3}ms samples={}",
            duration_ms(self.last_latency),
            average_latency_ns as f64 / 1_000_000.0,
            duration_ms(self.max_latency),
            self.sample_count,
        );

        self.interval_started_at = Instant::now();
        self.sample_count = 0;
        self.total_latency_ns = 0;
        self.max_latency = Duration::ZERO;
        self.last_latency = Duration::ZERO;
    }
}

#[cfg(not(debug_assertions))]
struct DevLatencyLogger;

#[cfg(not(debug_assertions))]
impl DevLatencyLogger {
    fn new() -> Self {
        Self
    }

    fn record(&mut self, _latency: Duration, _status: TransportStatus) {}

    fn flush(&mut self) {}
}

#[cfg(debug_assertions)]
fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconnect_probe_waits_for_interval_when_reader_is_missing() {
        assert!(!should_probe_reconnect(
            TransportStatus::WaitingForReader,
            RECONNECT_PROBE_INTERVAL - Duration::from_millis(1),
        ));
        assert!(should_probe_reconnect(
            TransportStatus::WaitingForReader,
            RECONNECT_PROBE_INTERVAL,
        ));
    }

    #[test]
    fn reconnect_probe_stays_disabled_while_transport_is_connected() {
        assert!(!should_probe_reconnect(
            TransportStatus::Connected,
            RECONNECT_PROBE_INTERVAL * 2,
        ));
        assert!(!should_probe_reconnect(
            TransportStatus::NewlyConnected,
            RECONNECT_PROBE_INTERVAL * 2,
        ));
    }

    #[test]
    fn wait_timeout_is_bounded_by_shutdown_budget_when_connected() {
        assert_eq!(
            next_wait_timeout(TransportStatus::Connected, Duration::ZERO),
            SHUTDOWN_WAIT_INTERVAL,
        );
        assert_eq!(
            next_wait_timeout(
                TransportStatus::NewlyConnected,
                RECONNECT_PROBE_INTERVAL * 2
            ),
            SHUTDOWN_WAIT_INTERVAL,
        );
    }

    #[test]
    fn wait_timeout_reaches_zero_when_reconnect_probe_is_due() {
        assert_eq!(
            next_wait_timeout(TransportStatus::WaitingForReader, Duration::ZERO),
            SHUTDOWN_WAIT_INTERVAL,
        );
        assert_eq!(
            next_wait_timeout(
                TransportStatus::WaitingForReader,
                RECONNECT_PROBE_INTERVAL - Duration::from_millis(5),
            ),
            Duration::from_millis(5),
        );
        assert_eq!(
            next_wait_timeout(TransportStatus::WaitingForReader, RECONNECT_PROBE_INTERVAL,),
            Duration::ZERO,
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    fn dev_latency_logger_ignores_waiting_for_reader_samples() {
        let mut logger = DevLatencyLogger::new();
        logger.record(Duration::from_millis(1), TransportStatus::WaitingForReader);

        assert_eq!(logger.sample_count, 0);
    }
}
