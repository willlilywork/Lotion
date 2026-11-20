//! DB worker (T0-R4): one thread owns the only connection; callers send typed,
//! validated commands over a channel and wait for the response.

use crate::{Pong, Workspace};
use serde::{Deserialize, Serialize};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

/// Every command the UI may send. Unknown commands and unknown fields are rejected
/// at deserialization, before anything reaches SQLite.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase", deny_unknown_fields)]
pub enum Command {
    // Struct form, not a unit variant: serde ignores extra fields on internally
    // tagged unit variants, which would defeat `deny_unknown_fields`.
    Ping {},
    #[cfg(test)]
    #[serde(skip)]
    Panic,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Response {
    Pong(Pong),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error, Serialize)]
#[serde(tag = "error", content = "message", rename_all = "camelCase")]
pub enum HostError {
    #[error("command failed: {0}")]
    Command(String),
    #[error("DB worker panicked; it was restarted and the command was not applied")]
    WorkerRestarted,
    #[error("DB worker is gone")]
    WorkerGone,
}

type Reply = mpsc::SyncSender<Result<Response, HostError>>;

/// Handle to the DB worker. Cheap to clone; every clone talks to the same thread.
#[derive(Clone)]
pub struct DbHost {
    tx: mpsc::Sender<(Command, Reply)>,
}

/// Where the worker's workspace lives.
#[derive(Debug, Clone)]
pub enum Location {
    File(PathBuf),
    Memory,
}

impl Location {
    fn open(&self) -> crate::Result<Workspace> {
        match self {
            Location::File(p) => Workspace::open(p),
            Location::Memory => Workspace::open_in_memory(),
        }
    }
}

impl DbHost {
    /// Opens the workspace on a new worker thread; fails if it cannot be opened.
    pub fn spawn(location: Location) -> crate::Result<Self> {
        let (tx, rx) = mpsc::channel::<(Command, Reply)>();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("lotion-db".into())
            .spawn(move || worker(location, rx, ready_tx))
            .expect("spawn DB worker thread");
        ready_rx
            .recv()
            .unwrap_or_else(|_| Err(crate::Error::WorkerFailed))?;
        Ok(Self { tx })
    }

    /// Sends one command and blocks until the worker answers.
    pub fn request(&self, cmd: Command) -> Result<Response, HostError> {
        let (reply_tx, reply_rx) = mpsc::sync_channel(1);
        self.tx
            .send((cmd, reply_tx))
            .map_err(|_| HostError::WorkerGone)?;
        reply_rx.recv().map_err(|_| HostError::WorkerGone)?
    }
}

fn worker(
    location: Location,
    rx: mpsc::Receiver<(Command, Reply)>,
    ready: mpsc::SyncSender<crate::Result<()>>,
) {
    let mut ws = match location.open() {
        Ok(ws) => {
            let _ = ready.send(Ok(()));
            ws
        }
        Err(e) => {
            let _ = ready.send(Err(e));
            return;
        }
    };
    for (cmd, reply) in rx {
        let result = catch_unwind(AssertUnwindSafe(|| apply(&ws, cmd)));
        let answer = match result {
            Ok(r) => r,
            Err(_) => {
                // The connection may be mid-transaction; drop it (SQLite rolls back)
                // and reopen once. If that fails, the worker stops.
                drop(ws);
                match location.open() {
                    Ok(fresh) => ws = fresh,
                    Err(_) => {
                        let _ = reply.send(Err(HostError::WorkerGone));
                        return;
                    }
                }
                Err(HostError::WorkerRestarted)
            }
        };
        let _ = reply.send(answer);
    }
}

fn apply(ws: &Workspace, cmd: Command) -> Result<Response, HostError> {
    match cmd {
        Command::Ping {} => ws
            .ping()
            .map(Response::Pong)
            .map_err(|e| HostError::Command(e.to_string())),
        #[cfg(test)]
        Command::Panic => panic!("test panic inside the DB worker"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: &str) -> Result<Command, serde_json::Error> {
        serde_json::from_str(json)
    }

    #[test]
    fn commands_are_validated() {
        assert_eq!(parse(r#"{"cmd":"ping"}"#).unwrap(), Command::Ping {});
        assert!(parse(r#"{"cmd":"dropTables"}"#).is_err());
        assert!(parse(r#"{"cmd":"ping","extra":1}"#).is_err());
        assert!(parse(r#"{"cmd":"panic"}"#).is_err());
        assert!(parse(r#"{}"#).is_err());
    }

    #[test]
    fn ping_round_trip_from_many_threads() {
        let host = DbHost::spawn(Location::Memory).unwrap();
        let handles: Vec<_> = (0..8)
            .map(|_| {
                let h = host.clone();
                thread::spawn(move || h.request(Command::Ping {}).unwrap())
            })
            .collect();
        for h in handles {
            let Response::Pong(p) = h.join().unwrap();
            assert_eq!(p.schema_version, crate::SCHEMA_VERSION);
        }
    }

    #[test]
    fn worker_survives_a_panic() {
        let host = DbHost::spawn(Location::Memory).unwrap();
        assert_eq!(
            host.request(Command::Panic),
            Err(HostError::WorkerRestarted)
        );
        assert!(matches!(
            host.request(Command::Ping {}),
            Ok(Response::Pong(_))
        ));
    }

    #[test]
    fn response_json_shape() {
        let host = DbHost::spawn(Location::Memory).unwrap();
        let json = serde_json::to_value(host.request(Command::Ping {}).unwrap()).unwrap();
        assert_eq!(json["kind"], "pong");
        assert_eq!(json["schemaVersion"], 1);
    }
}
