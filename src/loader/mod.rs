use crate::flow::{Flow, FlowKey, IPAddress};
use crate::parser::parse_pcap;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use tracing::{error, info, trace};

#[cfg(test)]
mod tests;

pub enum LoadStatus {
    Progress(f32),
    Loaded(HashMap<FlowKey, Flow>, Option<f64>, HashMap<IPAddress, Vec<String>>),
    Error(String),
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
                    error!(path = ?path_clone, error = ?e, "Failed to parse PCAP");
                    let _ = tx.send(LoadStatus::Error(e.to_string()));
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
    Error(String),
    Idle,
}

pub struct FlowLoadController {
    loader: Option<Loader>,
    last_progress: f32,
}

impl FlowLoadController {
    pub fn new(path: PathBuf) -> Self {
        Self {
            loader: Some(Loader::new(path)),
            last_progress: 0.0,
        }
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
                    error!(error = %error, "Loader encountered an error");
                    return FlowLoadStatus::Error(error);
                }
            }
        }

        status
    }
}
