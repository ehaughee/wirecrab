use crate::flow::{Flow, FlowKey, IPAddress};
use crate::parser::parse_pcap;
use anyhow::Error;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use tracing::{error, info, trace};

#[cfg(test)]
mod tests;

pub enum LoadStatus {
    Progress(f32),
    Loaded(
        HashMap<FlowKey, Flow>,
        Option<f64>,
        HashMap<IPAddress, Vec<String>>,
    ),
    Error(LoadError),
}

/// High level classification of common loader failures so the UI can craft better text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadErrorKind {
    NotFound,
    PermissionDenied,
    Other,
}

/// Detailed loader error that pairs the classification with the full diagnostic string.
#[derive(Clone, Debug)]
pub struct LoadError {
    kind: LoadErrorKind,
    message: String,
}

impl LoadError {
    pub fn new(kind: LoadErrorKind, message: String) -> Self {
        Self { kind, message }
    }

    pub fn kind(&self) -> LoadErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn summary<P: AsRef<Path>>(&self, path: P) -> String {
        let display = path.as_ref().display();
        match self.kind {
            LoadErrorKind::NotFound => format!("Capture {} was not found.", display),
            LoadErrorKind::PermissionDenied => {
                format!("Permission denied while opening {}.", display)
            }
            LoadErrorKind::Other => format!("Wirecrab could not open {}.", display),
        }
    }
}

fn categorize_loader_error(error: &Error) -> LoadErrorKind {
    for cause in error.chain() {
        if let Some(io_error) = cause.downcast_ref::<std::io::Error>() {
            return match io_error.kind() {
                ErrorKind::NotFound => LoadErrorKind::NotFound,
                ErrorKind::PermissionDenied => LoadErrorKind::PermissionDenied,
                _ => LoadErrorKind::Other,
            };
        }
    }
    LoadErrorKind::Other
}

pub struct Loader {
    rx: Receiver<LoadStatus>,
}

impl Loader {
    pub fn new(path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel();
        let path_clone = path.clone();
        info!(path = ?path_clone, "Spawning loader thread");
        thread::spawn(move || {
            let result = parse_pcap(&path_clone, |progress| {
                trace!(progress, "Parser progress update");
                let _ = tx.send(LoadStatus::Progress(progress));
            });

            match result {
                Ok((flows, start_ts, name_resolutions)) => {
                    info!(path = ?path_clone, flows = flows.len(), "PCAP parsed; sending results");
                    let _ = tx.send(LoadStatus::Loaded(flows, start_ts, name_resolutions));
                }
                Err(e) => {
                    let load_error = LoadError::new(categorize_loader_error(&e), e.to_string());
                    error!(
                        path = ?path_clone,
                        error = ?e,
                        "Failed to parse PCAP"
                    );
                    let _ = tx.send(LoadStatus::Error(load_error));
                }
            }
        });

        Self { rx }
    }

    pub fn try_recv(&self) -> Option<LoadStatus> {
        self.rx.try_recv().ok()
    }
}

pub enum FlowLoadStatus {
    Loading {
        progress: f32,
    },
    Ready {
        flows: HashMap<FlowKey, Flow>,
        start_timestamp: Option<f64>,
        name_resolutions: HashMap<IPAddress, Vec<String>>,
    },
    Error(LoadError),
    Idle,
}

pub struct FlowLoadController {
    loader: Option<Loader>,
    last_progress: f32,
}

impl FlowLoadController {
    pub fn new(path: PathBuf) -> Self {
        let mut controller = Self {
            loader: None,
            last_progress: 0.0,
        };
        controller.start(path);
        controller
    }

    /// Create an idle controller that can be started later.
    pub fn idle() -> Self {
        Self {
            loader: None,
            last_progress: 0.0,
        }
    }

    pub fn start(&mut self, path: PathBuf) {
        self.last_progress = 0.0;
        self.loader = Some(Loader::new(path));
    }

    pub fn poll(&mut self) -> FlowLoadStatus {
        if self.loader.is_none() {
            return FlowLoadStatus::Idle;
        }

        let mut status = FlowLoadStatus::Loading {
            progress: self.last_progress,
        };

        while let Some(message) = self.loader.as_ref().and_then(|loader| loader.try_recv()) {
            match message {
                LoadStatus::Progress(p) => {
                    self.last_progress = p;
                    trace!(progress = p, "Loader received progress update");
                    status = FlowLoadStatus::Loading { progress: p };
                }
                LoadStatus::Loaded(flows, start_timestamp, name_resolutions) => {
                    self.loader = None;
                    info!(flows = flows.len(), "Loader completed successfully");
                    return FlowLoadStatus::Ready {
                        flows,
                        start_timestamp,
                        name_resolutions,
                    };
                }
                LoadStatus::Error(error) => {
                    self.loader = None;
                    error!(
                        error = %error.message(),
                        kind = ?error.kind(),
                        "Loader encountered an error"
                    );
                    return FlowLoadStatus::Error(error);
                }
            }
        }

        status
    }
}
