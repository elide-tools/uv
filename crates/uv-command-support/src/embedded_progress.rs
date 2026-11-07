//! Progress and warning reporting hooks for hosts that embed uv as a library.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressUnit {
    None,
    Count,
    Bytes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgressOutcome {
    Success,
    Failure,
}

pub trait ProgressSink: Send + Sync + 'static {
    fn start(&self, _message: &str, _total: Option<u64>) {}

    fn update(&self, _position: Option<u64>, _total: Option<u64>, _message: Option<&str>) {}

    fn finish(&self, _outcome: ProgressOutcome, _message: &str) {}

    fn start_task(
        &self,
        _id: usize,
        _parent: Option<usize>,
        _message: &str,
        _total: Option<u64>,
        _unit: ProgressUnit,
    ) {
    }

    fn update_task(
        &self,
        _id: usize,
        _position: Option<u64>,
        _total: Option<u64>,
        _message: Option<&str>,
        _detail: Option<&str>,
    ) {
    }

    fn finish_task(&self, _id: usize, _outcome: ProgressOutcome, _message: &str) {}

    fn message(&self, _message: &str) {}
}

static ACTIVE: OnceLock<RwLock<Option<Arc<dyn ProgressSink>>>> = OnceLock::new();
static NEXT_TASK: AtomicUsize = AtomicUsize::new(1);

struct ProgressWarningSink {
    sink: Arc<dyn ProgressSink>,
}

impl uv_warnings::WarningSink for ProgressWarningSink {
    fn warning(&self, message: &str) {
        self.sink.message(message);
    }
}

fn active() -> &'static RwLock<Option<Arc<dyn ProgressSink>>> {
    ACTIVE.get_or_init(|| RwLock::new(None))
}

pub struct ProgressSinkGuard {
    previous: Option<Arc<dyn ProgressSink>>,
    _warning_guard: uv_warnings::WarningSinkGuard,
}

impl Drop for ProgressSinkGuard {
    fn drop(&mut self) {
        if let Ok(mut guard) = active().write() {
            *guard = self.previous.take();
        }
    }
}

pub fn install_progress_sink(sink: Arc<dyn ProgressSink>) -> ProgressSinkGuard {
    let warning_sink = Arc::new(ProgressWarningSink { sink: sink.clone() });
    let warning_guard = uv_warnings::install_warning_sink(warning_sink);
    let previous = active()
        .write()
        .ok()
        .and_then(|mut guard| guard.replace(sink));
    ProgressSinkGuard {
        previous,
        _warning_guard: warning_guard,
    }
}

pub fn has_progress_sink() -> bool {
    active().read().is_ok_and(|guard| guard.is_some())
}

pub fn with_progress_sink<R>(f: impl FnOnce(&dyn ProgressSink) -> R) -> Option<R> {
    let sink = active().read().ok().and_then(|guard| guard.clone())?;
    Some(f(sink.as_ref()))
}

pub fn start_task(
    parent: Option<usize>,
    message: &str,
    total: Option<u64>,
    unit: ProgressUnit,
) -> Option<usize> {
    let id = NEXT_TASK.fetch_add(1, Ordering::Relaxed);
    with_progress_sink(|sink| sink.start_task(id, parent, message, total, unit)).map(|()| id)
}

/// A top-level task reported to the installed [`ProgressSink`], if any.
#[derive(Debug, Default)]
pub struct EmbeddedTask {
    id: Option<usize>,
    position: AtomicU64,
    total: AtomicU64,
}

impl EmbeddedTask {
    /// Start a task; a no-op when no progress sink is installed.
    pub fn start(message: &str, total: Option<u64>, unit: ProgressUnit) -> Self {
        Self {
            id: start_task(None, message, total, unit),
            position: AtomicU64::new(0),
            total: AtomicU64::new(total.unwrap_or(0)),
        }
    }

    pub fn set_total(&self, total: u64) {
        self.total.store(total, Ordering::Relaxed);
        if let Some(id) = self.id {
            let position = self.position.load(Ordering::Relaxed);
            with_progress_sink(|sink| {
                sink.update_task(id, Some(position), Some(total), None, None)
            });
        }
    }

    pub fn inc(&self, message: Option<&str>) {
        if let Some(id) = self.id {
            let position = self.position.fetch_add(1, Ordering::Relaxed) + 1;
            let total = self.total.load(Ordering::Relaxed);
            let total = (total > 0).then_some(total);
            with_progress_sink(|sink| sink.update_task(id, Some(position), total, message, None));
        }
    }

    pub fn finish(&self, message: &str) {
        if let Some(id) = self.id {
            with_progress_sink(|sink| sink.finish_task(id, ProgressOutcome::Success, message));
        }
    }
}
