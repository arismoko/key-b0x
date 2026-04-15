use anyhow::Result;
use key_b0x_core::ControllerSnapshot;
use key_b0x_platform::{SlippiTransport, TransportStatus};

pub struct SnapshotEmitter<T: SlippiTransport> {
    transport: T,
    last_sent: Option<ControllerSnapshot>,
}

impl<T: SlippiTransport> SnapshotEmitter<T> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            last_sent: None,
        }
    }

    pub fn emit(&mut self, snapshot: &ControllerSnapshot) -> Result<TransportStatus> {
        let status = self.transport.ensure_connected()?;
        match status {
            TransportStatus::WaitingForReader => return Ok(status),
            TransportStatus::NewlyConnected => self.last_sent = None,
            TransportStatus::Connected => {}
        }

        let lines = if self.last_sent.is_none() {
            build_full_sync_lines(snapshot)
        } else {
            build_diff_lines(self.last_sent.as_ref().unwrap(), snapshot)
        };

        for line in &lines {
            let line_status = self.transport.send_line(line)?;
            if line_status == TransportStatus::WaitingForReader {
                self.last_sent = None;
                return Ok(line_status);
            }
        }

        self.last_sent = Some(snapshot.clone());
        Ok(status)
    }
}

const BUTTONS: [(&str, fn(&ControllerSnapshot) -> bool); 12] = [
    ("A", |snapshot| snapshot.a),
    ("B", |snapshot| snapshot.b),
    ("X", |snapshot| snapshot.x),
    ("Y", |snapshot| snapshot.y),
    ("Z", |snapshot| snapshot.z),
    ("START", |snapshot| snapshot.start),
    ("L", |snapshot| snapshot.l),
    ("R", |snapshot| snapshot.r),
    ("D_UP", |snapshot| snapshot.d_up),
    ("D_DOWN", |snapshot| snapshot.d_down),
    ("D_LEFT", |snapshot| snapshot.d_left),
    ("D_RIGHT", |snapshot| snapshot.d_right),
];

fn build_full_sync_lines(snapshot: &ControllerSnapshot) -> Vec<String> {
    let mut lines = Vec::with_capacity(16);
    for (token, current) in BUTTONS.map(|(token, getter)| (token, getter(snapshot))) {
        lines.push(if current {
            format!("PRESS {token}")
        } else {
            format!("RELEASE {token}")
        });
    }

    lines.push(format!(
        "SET MAIN {:.6} {:.6}",
        snapshot.main_x, snapshot.main_y
    ));
    lines.push(format!("SET C {:.6} {:.6}", snapshot.c_x, snapshot.c_y));
    lines.push(format!("SET L {:.6}", snapshot.l_analog));
    lines.push(format!("SET R {:.6}", snapshot.r_analog));
    lines
}

fn build_diff_lines(previous: &ControllerSnapshot, current: &ControllerSnapshot) -> Vec<String> {
    let mut lines = Vec::new();

    for ((token, prev), (_, next)) in BUTTONS
        .map(|(token, getter)| (token, getter(previous)))
        .into_iter()
        .zip(BUTTONS.map(|(token, getter)| (token, getter(current))))
    {
        if prev != next {
            lines.push(if next {
                format!("PRESS {token}")
            } else {
                format!("RELEASE {token}")
            });
        }
    }

    if axes_changed(previous.main_x, current.main_x)
        || axes_changed(previous.main_y, current.main_y)
    {
        lines.push(format!(
            "SET MAIN {:.6} {:.6}",
            current.main_x, current.main_y
        ));
    }
    if axes_changed(previous.c_x, current.c_x) || axes_changed(previous.c_y, current.c_y) {
        lines.push(format!("SET C {:.6} {:.6}", current.c_x, current.c_y));
    }
    if axes_changed(previous.l_analog, current.l_analog) {
        lines.push(format!("SET L {:.6}", current.l_analog));
    }
    if axes_changed(previous.r_analog, current.r_analog) {
        lines.push(format!("SET R {:.6}", current.r_analog));
    }

    lines
}

fn axes_changed(lhs: f64, rhs: f64) -> bool {
    (lhs - rhs).abs() > 1e-9
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MemoryTransport {
        connected: bool,
        lines: Vec<String>,
        next_status: TransportStatus,
    }

    impl MemoryTransport {
        fn with_status(status: TransportStatus) -> Self {
            Self {
                connected: status != TransportStatus::WaitingForReader,
                lines: Vec::new(),
                next_status: status,
            }
        }
    }

    impl SlippiTransport for MemoryTransport {
        fn ensure_connected(&mut self) -> Result<TransportStatus> {
            let status = self.next_status;
            if status == TransportStatus::NewlyConnected {
                self.next_status = TransportStatus::Connected;
            }
            Ok(status)
        }

        fn send_line(&mut self, line: &str) -> Result<TransportStatus> {
            self.lines.push(line.to_string());
            Ok(if self.connected {
                TransportStatus::Connected
            } else {
                self.next_status
            })
        }
    }

    #[test]
    fn full_sync_sends_buttons_and_axes() {
        let mut transport = MemoryTransport::with_status(TransportStatus::NewlyConnected);
        transport.connected = true;
        let mut emitter = SnapshotEmitter::new(transport);

        let snapshot = ControllerSnapshot {
            a: true,
            main_x: 1.0,
            main_y: 0.5,
            c_x: 0.5,
            c_y: 0.5,
            ..ControllerSnapshot::neutral()
        };
        emitter.emit(&snapshot).unwrap();

        assert!(emitter.transport.lines.contains(&"PRESS A".to_string()));
        assert!(
            emitter
                .transport
                .lines
                .contains(&"SET MAIN 1.000000 0.500000".to_string())
        );
    }

    #[test]
    fn diff_only_sends_changed_values() {
        let mut transport = MemoryTransport::with_status(TransportStatus::NewlyConnected);
        transport.connected = true;
        let mut emitter = SnapshotEmitter::new(transport);

        let first = ControllerSnapshot {
            a: true,
            ..ControllerSnapshot::neutral()
        };
        emitter.emit(&first).unwrap();
        emitter.transport.lines.clear();

        let second = ControllerSnapshot {
            a: true,
            b: true,
            ..ControllerSnapshot::neutral()
        };
        emitter.emit(&second).unwrap();
        assert_eq!(emitter.transport.lines, vec!["PRESS B".to_string()]);
    }

    #[test]
    fn waiting_for_reader_skips_snapshot_state() {
        let transport = MemoryTransport::with_status(TransportStatus::WaitingForReader);
        let mut emitter = SnapshotEmitter::new(transport);
        let snapshot = ControllerSnapshot {
            a: true,
            ..ControllerSnapshot::neutral()
        };

        let status = emitter.emit(&snapshot).unwrap();
        assert_eq!(status, TransportStatus::WaitingForReader);
        assert!(emitter.last_sent.is_none());
    }
}
