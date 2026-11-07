use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use uv_command_support::Printer;
use uv_command_support::embedded_progress::{self, EmbeddedTask, ProgressUnit};
use uv_command_support::progress::{ProgressReporter, progress_target};
use uv_distribution_types::BuildableSource;
use uv_distribution_types::CachedDist;
use uv_normalize::PackageName;
use uv_redacted::SafeUrl;

#[derive(Debug)]
pub(super) struct PrepareReporter {
    reporter: ProgressReporter,
}

impl From<Printer> for PrepareReporter {
    fn from(printer: Printer) -> Self {
        let multi_progress = MultiProgress::with_draw_target(progress_target(printer));
        let root = multi_progress.add(ProgressBar::with_draw_target(
            None,
            progress_target(printer),
        ));
        root.enable_steady_tick(Duration::from_millis(200));
        root.set_style(
            ProgressStyle::with_template("{spinner:.white} {msg:.dim} ({pos}/{len})")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        root.set_message("Preparing packages...");

        let embedded_root =
            embedded_progress::start_task(None, "Preparing packages", None, ProgressUnit::Count);
        let reporter =
            ProgressReporter::new(root, multi_progress, printer).with_embedded_root(embedded_root);
        Self { reporter }
    }
}

impl PrepareReporter {
    #[must_use]
    pub(super) fn with_length(self, length: u64) -> Self {
        self.reporter.root.set_length(length);
        self.reporter.set_root_length(length);
        self
    }
}

impl uv_installer::PrepareReporter for PrepareReporter {
    fn on_progress(&self, _dist: &CachedDist) {
        self.reporter.root.inc(1);
        self.reporter.inc_root(None);
    }

    fn on_complete(&self) {
        // Need an extra call to `set_message` here to fully clear avoid leaving ghost output
        // in Jupyter notebooks.
        self.reporter.root.set_message("");
        self.reporter.root.finish_and_clear();
        self.reporter.finish_root("Prepared packages");
    }

    fn on_build_start(&self, source: &BuildableSource) -> usize {
        self.reporter.on_build_start(&source.color_display())
    }

    fn on_build_complete(&self, source: &BuildableSource, id: usize) {
        self.reporter.on_build_complete(&source.color_display(), id);
    }

    fn on_download_start(&self, name: &PackageName, size: Option<u64>) -> usize {
        self.reporter.on_download_start(name.to_string(), size)
    }

    fn on_download_progress(&self, id: usize, bytes: u64) {
        self.reporter.on_download_progress(id, bytes);
    }

    fn on_download_complete(&self, _name: &PackageName, id: usize) {
        self.reporter.on_download_complete(id);
    }

    fn on_checkout_start(&self, url: &SafeUrl, rev: &str) -> usize {
        self.reporter.on_checkout_start(url, rev)
    }

    fn on_checkout_complete(&self, url: &SafeUrl, rev: &str, id: usize) {
        self.reporter.on_checkout_complete(url, rev, id);
    }
}

#[derive(Debug)]
pub(super) struct InstallReporter {
    progress: ProgressBar,
    embedded: EmbeddedTask,
}

impl From<Printer> for InstallReporter {
    fn from(printer: Printer) -> Self {
        let progress = ProgressBar::with_draw_target(None, progress_target(printer));
        progress.set_style(
            ProgressStyle::with_template("{bar:20} [{pos}/{len}] {wide_msg:.dim}").unwrap(),
        );
        progress.set_message("Installing Python packages...");
        let embedded = EmbeddedTask::start("Installing Python packages", None, ProgressUnit::Count);
        Self { progress, embedded }
    }
}

impl InstallReporter {
    #[must_use]
    pub(super) fn with_length(self, length: u64) -> Self {
        self.progress.set_length(length);
        self.embedded.set_total(length);
        self
    }
}

impl uv_installer::InstallReporter for InstallReporter {
    fn on_install_progress(&self, wheel: &CachedDist) {
        let message = format!("{wheel}");
        self.progress.set_message(message.clone());
        self.progress.inc(1);
        self.embedded.inc(Some(&message));
    }

    fn on_install_complete(&self) {
        self.progress.set_message("");
        self.progress.finish_and_clear();
        self.embedded.finish("Python packages installed");
    }
}
