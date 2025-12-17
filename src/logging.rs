use anyhow::{Context, Result};
use std::fs::OpenOptions;
use std::path::Path;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::{self, WorkerGuard};
use tracing_subscriber::EnvFilter;

pub struct LoggingGuard {
    _worker: Option<WorkerGuard>,
}

impl LoggingGuard {
    pub fn none() -> Self {
        Self { _worker: None }
    }

    pub fn with_guard(guard: WorkerGuard) -> Self {
        Self {
            _worker: Some(guard),
        }
    }
}

pub fn init_logging(to_stdout: bool, file_path: &Path, level: LevelFilter) -> Result<LoggingGuard> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env_lossy();

    if to_stdout {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_target(true)
            .with_thread_ids(true)
            .init();
        Ok(LoggingGuard::none())
    } else {
        if let Some(parent) = file_path.parent() && !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create log directory {parent:?}"))?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .with_context(|| format!("Failed to open log file {file_path:?}"))?;

        let (writer, guard) = non_blocking::NonBlockingBuilder::default().finish(file);

        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(writer)
            .with_target(true)
            .with_thread_ids(true)
            .init();

        Ok(LoggingGuard::with_guard(guard))
    }
}
