use crate::flow::{Flow, FlowKey};
use crate::parser;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

pub enum LoadStatus {
    Progress(f32),
    Loaded(HashMap<FlowKey, Flow>, Option<f64>),
    Error(String),
}

pub struct Loader {
    rx: Receiver<LoadStatus>,
}

impl Loader {
    pub fn new(path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel();
        let path_clone = path.clone();
        thread::spawn(move || {
            let result = parser::parse_pcap(&path_clone, |progress| {
                let _ = tx.send(LoadStatus::Progress(progress));
            });

            match result {
                Ok((flows, start_ts)) => {
                    let _ = tx.send(LoadStatus::Loaded(flows, start_ts));
                }
                Err(e) => {
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

        loop {
            let next = match self.loader.as_ref().and_then(|loader| loader.try_recv()) {
                Some(status) => status,
                None => break,
            };

            match next {
                LoadStatus::Progress(p) => {
                    self.last_progress = p;
                    status = FlowLoadStatus::Loading { progress: p };
                }
                LoadStatus::Loaded(flows, start_timestamp) => {
                    self.loader = None;
                    return FlowLoadStatus::Ready {
                        flows,
                        start_timestamp,
                    };
                }
                LoadStatus::Error(error) => {
                    self.loader = None;
                    return FlowLoadStatus::Error(error);
                }
            }
        }

        status
    }
}
