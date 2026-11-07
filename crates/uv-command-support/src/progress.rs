use std::env;
use std::fmt::{self, Write};
use std::ops::Deref;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};

use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use owo_colors::OwoColorize;
use rustc_hash::FxHashMap;
use uv_console::human_readable_bytes;
use uv_redacted::SafeUrl;
use uv_static::EnvVars;

use crate::Printer;
use crate::embedded_progress::{self, ProgressOutcome, ProgressUnit};

/// Since downloads, fetches and builds run in parallel, their message output order is
/// non-deterministic, so can't capture them in test output.
static HAS_UV_INTERNAL__TEST_NO_CLI_PROGRESS: LazyLock<bool> =
    LazyLock::new(|| env::var(EnvVars::UV_INTERNAL__TEST_NO_CLI_PROGRESS).is_ok());

/// The draw target for progress bars: hidden while an embedding host's progress sink is
/// installed, since the host renders progress itself.
pub fn progress_target(printer: Printer) -> ProgressDrawTarget {
    if embedded_progress::has_progress_sink() {
        ProgressDrawTarget::hidden()
    } else {
        printer.target()
    }
}

#[derive(Debug)]
pub struct ProgressReporter {
    pub printer: Printer,
    pub root: ProgressBar,
    embedded_root: Option<usize>,
    embedded_root_position: AtomicU64,
    embedded_root_total: AtomicU64,
    embedded_task_positions: Mutex<FxHashMap<usize, u64>>,
    mode: ProgressMode,
}

#[derive(Debug)]
enum ProgressMode {
    /// Reports top-level progress.
    Single,
    /// Reports progress of all concurrent download, build, and checkout processes.
    Multi {
        multi_progress: MultiProgress,
        state: Arc<Mutex<BarState>>,
    },
}

#[derive(Debug)]
enum ProgressBarKind {
    /// A progress bar with an increasing value, such as a download.
    Numeric {
        progress: ProgressBar,
        /// The download size in bytes, if known.
        size: Option<u64>,
    },
    /// A progress spinner for a task, such as a build.
    Spinner { progress: ProgressBar },
}

impl Deref for ProgressBarKind {
    type Target = ProgressBar;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Numeric { progress, .. } => progress,
            Self::Spinner { progress } => progress,
        }
    }
}

#[derive(Debug)]
struct BarState {
    /// The number of bars that precede any download bars (i.e., build/checkout status).
    headers: usize,
    /// A list of download bar sizes, in descending order.
    sizes: Vec<u64>,
    /// A map of progress bars, by ID.
    bars: FxHashMap<usize, ProgressBarKind>,
    /// A monotonic counter for bar IDs.
    id: usize,
    /// The maximum length of all bar names encountered.
    max_len: usize,
}

impl Default for BarState {
    fn default() -> Self {
        Self {
            headers: 0,
            sizes: Vec::default(),
            bars: FxHashMap::default(),
            id: 0,
            // Avoid resizing the progress bar templates too often by starting with a padding
            // that's wider than most package names.
            max_len: 20,
        }
    }
}

impl BarState {
    /// Returns a unique ID for a new progress bar.
    fn id(&mut self) -> usize {
        self.id += 1;
        self.id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Upload,
    Download,
    Extract,
    Hash,
}

impl Direction {
    fn as_str(&self) -> &str {
        match self {
            Self::Download => "Downloading",
            Self::Upload => "Uploading",
            Self::Extract => "Extracting",
            Self::Hash => "Hashing",
        }
    }
}

impl ProgressReporter {
    pub fn new(root: ProgressBar, multi_progress: MultiProgress, printer: Printer) -> Self {
        let mode = if env::var(EnvVars::JPY_SESSION_NAME).is_ok() {
            // Disable concurrent progress bars when running inside a Jupyter notebook
            // because the Jupyter terminal does not support clearing previous lines.
            // See: https://github.com/astral-sh/uv/issues/3887.
            ProgressMode::Single
        } else {
            ProgressMode::Multi {
                state: Arc::default(),
                multi_progress,
            }
        };

        Self {
            printer,
            root,
            embedded_root: None,
            embedded_root_position: AtomicU64::new(0),
            embedded_root_total: AtomicU64::new(0),
            embedded_task_positions: Mutex::new(FxHashMap::default()),
            mode,
        }
    }

    /// Report this reporter's progress as the given embedded-host task, if any.
    #[must_use]
    pub fn with_embedded_root(mut self, embedded_root: Option<usize>) -> Self {
        self.embedded_root = embedded_root;
        self
    }

    /// Stop reporting this reporter's progress to an embedding host.
    #[must_use]
    pub fn without_embedded_root(mut self) -> Self {
        self.embedded_root = None;
        self
    }

    pub fn set_root_length(&self, length: u64) {
        self.embedded_root_total.store(length, Ordering::Relaxed);
        if let Some(id) = self.embedded_root {
            let position = self.embedded_root_position.load(Ordering::Relaxed);
            embedded_progress::with_progress_sink(|sink| {
                sink.update_task(id, Some(position), Some(length), None, None);
            });
        }
    }

    pub fn set_root_message(&self, message: String) {
        if let Some(id) = self.embedded_root {
            let position = self.embedded_root_position.load(Ordering::Relaxed);
            let total = self.embedded_total();
            embedded_progress::with_progress_sink(|sink| {
                sink.update_task(id, Some(position), total, Some(&message), None);
            });
        }
    }

    pub fn inc_root(&self, message: Option<String>) {
        if let Some(id) = self.embedded_root {
            let position = self.embedded_root_position.fetch_add(1, Ordering::Relaxed) + 1;
            let total = self.embedded_total();
            embedded_progress::with_progress_sink(|sink| {
                sink.update_task(id, Some(position), total, message.as_deref(), None);
            });
        }
    }

    pub fn finish_root(&self, message: &str) {
        if let Some(id) = self.embedded_root {
            embedded_progress::with_progress_sink(|sink| {
                sink.finish_task(id, ProgressOutcome::Success, message);
            });
        }
    }

    fn embedded_total(&self) -> Option<u64> {
        let total = self.embedded_root_total.load(Ordering::Relaxed);
        (total > 0).then_some(total)
    }

    fn hidden_fallback(&self, multi_progress: &MultiProgress) -> bool {
        !embedded_progress::has_progress_sink()
            && multi_progress.is_hidden()
            && !*HAS_UV_INTERNAL__TEST_NO_CLI_PROGRESS
    }

    fn start_embedded_task(
        &self,
        message: &str,
        total: Option<u64>,
        unit: ProgressUnit,
    ) -> Option<usize> {
        let parent = self.embedded_root?;
        let id = embedded_progress::start_task(Some(parent), message, total, unit)?;
        if let Ok(mut positions) = self.embedded_task_positions.lock() {
            positions.insert(id, 0);
        }
        Some(id)
    }

    fn inc_embedded_task(&self, id: usize, inc: u64) {
        let position = self
            .embedded_task_positions
            .lock()
            .ok()
            .map(|mut positions| {
                let position = positions.entry(id).or_insert(0);
                *position = position.saturating_add(inc);
                *position
            });
        embedded_progress::with_progress_sink(|sink| {
            sink.update_task(id, position, None, None, None);
        });
    }

    fn finish_embedded_task(&self, id: usize, outcome: ProgressOutcome, message: &str) {
        if let Ok(mut positions) = self.embedded_task_positions.lock() {
            positions.remove(&id);
        }
        embedded_progress::with_progress_sink(|sink| {
            sink.finish_task(id, outcome, message);
        });
    }

    /// Start reporting a build using the caller's source display.
    pub fn on_build_start(&self, source: &dyn fmt::Display) -> usize {
        if self.embedded_root.is_some() {
            return self
                .start_embedded_task(&format!("Building {source}"), None, ProgressUnit::None)
                .unwrap_or(0);
        }

        let ProgressMode::Multi {
            multi_progress,
            state,
        } = &self.mode
        else {
            return 0;
        };

        let mut state = state.lock().unwrap();
        let id = state.id();

        let progress = multi_progress.insert_before(
            &self.root,
            ProgressBar::with_draw_target(None, progress_target(self.printer)),
        );

        progress.set_style(ProgressStyle::with_template("{wide_msg}").unwrap());
        let message = format!("   {} {}", "Building".bold().cyan(), source);
        if self.hidden_fallback(multi_progress) {
            let _ = writeln!(self.printer.stderr(), "{message}");
        }
        progress.set_message(message);

        state.headers += 1;
        state.bars.insert(id, ProgressBarKind::Spinner { progress });
        id
    }

    /// Finish reporting a build using the caller's source display.
    pub fn on_build_complete(&self, source: &dyn fmt::Display, id: usize) {
        if self.embedded_root.is_some() {
            self.finish_embedded_task(id, ProgressOutcome::Success, &format!("Built {source}"));
            return;
        }

        let ProgressMode::Multi {
            state,
            multi_progress,
        } = &self.mode
        else {
            return;
        };

        let progress = {
            let mut state = state.lock().unwrap();
            state.headers -= 1;
            state.bars.remove(&id).unwrap()
        };

        let message = format!("      {} {}", "Built".bold().green(), source);
        if self.hidden_fallback(multi_progress) {
            let _ = writeln!(self.printer.stderr(), "{message}");
        }
        progress.finish_with_message(message);
    }

    pub fn on_request_start(&self, direction: Direction, name: String, size: Option<u64>) -> usize {
        if self.embedded_root.is_some() {
            return self
                .start_embedded_task(
                    &format!("{} {name}", direction.as_str()),
                    size,
                    ProgressUnit::Bytes,
                )
                .unwrap_or(0);
        }

        let ProgressMode::Multi {
            multi_progress,
            state,
        } = &self.mode
        else {
            return 0;
        };

        let mut state = state.lock().unwrap();

        // Preserve ascending order.
        let position = size.map_or(0, |size| state.sizes.partition_point(|&len| len < size));
        state.sizes.insert(position, size.unwrap_or(0));
        state.max_len = std::cmp::max(state.max_len, name.len());

        let max_len = state.max_len;
        for progress in state.bars.values_mut() {
            // Ignore spinners, such as for builds.
            if let ProgressBarKind::Numeric { progress, .. } = progress {
                let template = format!(
                    "{{msg:{max_len}.dim}} {{bar:30.green/black.dim}} {{binary_bytes:>7}}/{{binary_total_bytes:7}}"
                );
                progress.set_style(
                    ProgressStyle::with_template(&template)
                        .unwrap()
                        .progress_chars("--"),
                );
                progress.tick();
            }
        }

        let progress = multi_progress.insert(
            // Make sure not to reorder the initial "Preparing..." bar, or any previous bars.
            position + 1 + state.headers,
            ProgressBar::with_draw_target(size, progress_target(self.printer)),
        );

        if let Some(size) = size {
            // We're using binary bytes to match `human_readable_bytes`.
            progress.set_style(
                ProgressStyle::with_template(
                    &format!(
                        "{{msg:{}.dim}} {{bar:30.green/black.dim}} {{binary_bytes:>7}}/{{binary_total_bytes:7}}", state.max_len
                    ),
                )
                    .unwrap()
                    .progress_chars("--"),
            );
            // If the file is larger than 1MB, show a message to indicate that this may take
            // a while keeping the log concise.
            if self.hidden_fallback(multi_progress) && size > 1024 * 1024 {
                let _ = writeln!(
                    self.printer.stderr(),
                    "{} {} {}",
                    direction.as_str().bold().cyan(),
                    name,
                    format!("({:.1})", human_readable_bytes(size)).dimmed()
                );
            }
            progress.set_message(name);
        } else {
            progress.set_style(ProgressStyle::with_template("{wide_msg:.dim} ....").unwrap());
            if self.hidden_fallback(multi_progress) {
                let _ = writeln!(
                    self.printer.stderr(),
                    "{} {}",
                    direction.as_str().bold().cyan(),
                    name
                );
            }
            progress.set_message(name);
            progress.finish();
        }

        let id = state.id();
        state
            .bars
            .insert(id, ProgressBarKind::Numeric { progress, size });
        id
    }

    pub fn on_request_progress(&self, id: usize, bytes: u64) {
        if self.embedded_root.is_some() {
            self.inc_embedded_task(id, bytes);
            return;
        }

        let ProgressMode::Multi { state, .. } = &self.mode else {
            return;
        };

        // Avoid panics due to reads on failed requests.
        // https://github.com/astral-sh/uv/issues/17090
        // TODO(konsti): Add a debug assert once https://github.com/seanmonstar/reqwest/issues/2884
        // is fixed
        if let Some(bar) = state.lock().unwrap().bars.get(&id) {
            bar.inc(bytes);
        }
    }

    pub fn on_request_complete(&self, direction: Direction, id: usize) {
        if self.embedded_root.is_some() {
            let message = match direction {
                Direction::Download => "Downloaded",
                Direction::Upload => "Uploaded",
                Direction::Extract => "Extracted",
                Direction::Hash => "Hashed",
            };
            self.finish_embedded_task(id, ProgressOutcome::Success, message);
            return;
        }

        let ProgressMode::Multi {
            state,
            multi_progress,
        } = &self.mode
        else {
            return;
        };

        let mut state = state.lock().unwrap();
        if let ProgressBarKind::Numeric { progress, size } = state.bars.remove(&id).unwrap() {
            if self.hidden_fallback(multi_progress) && size.is_none_or(|size| size > 1024 * 1024) {
                let _ = writeln!(
                    self.printer.stderr(),
                    " {} {}",
                    match direction {
                        Direction::Download => "Downloaded",
                        Direction::Upload => "Uploaded",
                        Direction::Extract => "Extracted",
                        Direction::Hash => "Hashed",
                    }
                    .bold()
                    .cyan(),
                    progress.message()
                );
            }
            progress.finish_and_clear();
        } else {
            debug_assert!(false, "Request progress bars are numeric");
        }
    }

    pub fn on_download_progress(&self, id: usize, bytes: u64) {
        self.on_request_progress(id, bytes);
    }

    pub fn on_download_complete(&self, id: usize) {
        self.on_request_complete(Direction::Download, id);
    }

    pub fn on_download_start(&self, name: String, size: Option<u64>) -> usize {
        self.on_request_start(Direction::Download, name, size)
    }

    pub fn on_upload_progress(&self, id: usize, bytes: u64) {
        self.on_request_progress(id, bytes);
    }

    pub fn on_upload_complete(&self, id: usize) {
        self.on_request_complete(Direction::Upload, id);
    }

    pub fn on_upload_start(&self, name: String, size: Option<u64>) -> usize {
        self.on_request_start(Direction::Upload, name, size)
    }

    pub fn on_hash_progress(&self, id: usize, bytes: u64) {
        self.on_request_progress(id, bytes);
    }

    pub fn on_hash_complete(&self, id: usize) {
        self.on_request_complete(Direction::Hash, id);
    }

    pub fn on_hash_start(&self, name: String, size: Option<u64>) -> usize {
        self.on_request_start(Direction::Hash, name, size)
    }

    pub fn on_checkout_start(&self, url: &SafeUrl, rev: &str) -> usize {
        if self.embedded_root.is_some() {
            return self
                .start_embedded_task(&format!("Updating {url} ({rev})"), None, ProgressUnit::None)
                .unwrap_or(0);
        }

        let ProgressMode::Multi {
            multi_progress,
            state,
        } = &self.mode
        else {
            return 0;
        };

        let mut state = state.lock().unwrap();
        let id = state.id();

        let progress = multi_progress.insert_before(
            &self.root,
            ProgressBar::with_draw_target(None, progress_target(self.printer)),
        );

        progress.set_style(ProgressStyle::with_template("{wide_msg}").unwrap());
        let message = format!("   {} {} ({})", "Updating".bold().cyan(), url, rev.dimmed());
        if self.hidden_fallback(multi_progress) {
            let _ = writeln!(self.printer.stderr(), "{message}");
        }
        progress.set_message(message);
        progress.finish();

        state.headers += 1;
        state.bars.insert(id, ProgressBarKind::Spinner { progress });
        id
    }

    pub fn on_checkout_complete(&self, url: &SafeUrl, rev: &str, id: usize) {
        if self.embedded_root.is_some() {
            self.finish_embedded_task(
                id,
                ProgressOutcome::Success,
                &format!("Updated {url} ({rev})"),
            );
            return;
        }

        let ProgressMode::Multi {
            state,
            multi_progress,
        } = &self.mode
        else {
            return;
        };

        let progress = {
            let mut state = state.lock().unwrap();
            state.headers -= 1;
            state.bars.remove(&id).unwrap()
        };

        let message = format!(
            "    {} {} ({})",
            "Updated".bold().green(),
            url,
            rev.dimmed()
        );
        if self.hidden_fallback(multi_progress) {
            let _ = writeln!(self.printer.stderr(), "{message}");
        }
        progress.finish_with_message(message);
    }
}
