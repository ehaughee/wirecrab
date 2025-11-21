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
