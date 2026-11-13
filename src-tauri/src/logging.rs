//! Logging: to the terminal during development, and to a daily file in the app log folder
//! (the installed app has no console). The token is never logged at any level.

use std::path::Path;

use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

/// Default filter; override with `RUST_LOG`.
const DEFAULT_FILTER: &str = "info,webeep_sync_lib=debug,wry=warn,tauri=warn";
const FILE_PREFIX: &str = "webeep-sync";
const FILE_SUFFIX: &str = "log";
/// One file per day, a week of history.
const KEPT_FILES: usize = 7;

/// Keeps the file writer alive: dropping it flushes and stops file logging.
pub struct LogGuard {
    _file: Option<WorkerGuard>,
}

fn filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER))
}

/// Starts logging. If the log folder is unusable, logs go to the terminal only.
pub fn init(log_dir: Option<&Path>) -> LogGuard {
    let (writer, guard) = log_dir.and_then(file_writer).unzip();
    let file_layer = writer.map(|w| {
        fmt::layer()
            .with_ansi(false)
            .with_writer(w)
            .with_filter(filter())
    });
    // Fails only if a global subscriber is already set, which is harmless.
    let _ = tracing_subscriber::registry()
        .with(fmt::layer().with_filter(filter()))
        .with(file_layer)
        .try_init();
    LogGuard { _file: guard }
}

fn file_writer(dir: &Path) -> Option<(NonBlocking, WorkerGuard)> {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(FILE_PREFIX)
        .filename_suffix(FILE_SUFFIX)
        .max_log_files(KEPT_FILES)
        .build(dir)
        .ok()?;
    Some(tracing_appender::non_blocking(appender))
}
