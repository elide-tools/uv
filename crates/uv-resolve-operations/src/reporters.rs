use std::time::Duration;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use uv_command_support::Printer;
use uv_command_support::embedded_progress::{self, EmbeddedTask, ProgressUnit};
use uv_command_support::progress::{ProgressReporter, progress_target};
use uv_distribution_types::BuildableSource;
use uv_distribution_types::VersionOrUrlRef;
use uv_normalize::PackageName;
use uv_pep440::Version;
use uv_redacted::SafeUrl;

#[derive(Debug)]
pub struct ResolverReporter {
    reporter: ProgressReporter,
}

impl ResolverReporter {
    #[must_use]
    pub(super) fn with_length(self, length: u64) -> Self {
        self.reporter.root.set_length(length);
        self.reporter.set_root_length(length);
        self
    }

    /// Don't report this resolution to an embedding host as a top-level task.
    #[must_use]
    pub(super) fn without_embedded_root(self) -> Self {
        Self {
            reporter: self.reporter.without_embedded_root(),
        }
    }
}

impl From<Printer> for ResolverReporter {
    fn from(printer: Printer) -> Self {
        let multi_progress = MultiProgress::with_draw_target(progress_target(printer));
        let root = multi_progress.add(ProgressBar::with_draw_target(
            None,
            progress_target(printer),
        ));
        root.enable_steady_tick(Duration::from_millis(200));
        root.set_style(
            ProgressStyle::with_template("{spinner:.white} {wide_msg:.dim}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        root.set_message("Resolving Python dependencies...");

        let embedded_root = embedded_progress::start_task(
            None,
            "Resolving Python dependencies",
            None,
            ProgressUnit::Count,
        );
        let reporter =
            ProgressReporter::new(root, multi_progress, printer).with_embedded_root(embedded_root);
        Self { reporter }
    }
}

impl uv_resolver::ResolverReporter for ResolverReporter {
    fn on_progress(&self, name: &PackageName, version_or_url: &VersionOrUrlRef) {
        match version_or_url {
            VersionOrUrlRef::Version(version) => {
                let message = format!("{name}=={version}");
                self.reporter.root.set_message(message.clone());
                self.reporter.set_root_message(message);
            }
            VersionOrUrlRef::Url(url) => {
                let message = format!("{name} @ {url}");
                self.reporter.root.set_message(message.clone());
                self.reporter.set_root_message(message);
            }
        }
    }

    fn on_complete(&self) {
        self.reporter.root.set_message("");
        self.reporter.root.finish_and_clear();
        self.reporter.finish_root("Python dependencies resolved");
    }

    fn on_build_start(&self, source: &BuildableSource) -> usize {
        self.reporter.on_build_start(&source.color_display())
    }

    fn on_build_complete(&self, source: &BuildableSource, id: usize) {
        self.reporter.on_build_complete(&source.color_display(), id);
    }

    fn on_checkout_start(&self, url: &SafeUrl, rev: &str) -> usize {
        self.reporter.on_checkout_start(url, rev)
    }

    fn on_checkout_complete(&self, url: &SafeUrl, rev: &str, id: usize) {
        self.reporter.on_checkout_complete(url, rev, id);
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
}

impl uv_distribution::Reporter for ResolverReporter {
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
pub struct LatestVersionReporter {
    progress: ProgressBar,
    embedded: EmbeddedTask,
}

impl From<Printer> for LatestVersionReporter {
    fn from(printer: Printer) -> Self {
        let progress = ProgressBar::with_draw_target(None, progress_target(printer));
        progress.set_style(
            ProgressStyle::with_template("{bar:20} [{pos}/{len}] {wide_msg:.dim}").unwrap(),
        );
        progress.set_message("Fetching latest versions...");
        let embedded = EmbeddedTask::start("Fetching latest versions", None, ProgressUnit::Count);
        Self { progress, embedded }
    }
}

impl LatestVersionReporter {
    #[must_use]
    pub fn with_length(self, length: u64) -> Self {
        self.progress.set_length(length);
        self.embedded.set_total(length);
        self
    }

    pub fn on_fetch_progress(&self) {
        self.progress.inc(1);
        self.embedded.inc(None);
    }

    pub fn on_fetch_version(&self, name: &PackageName, version: &Version) {
        let message = format!("{name} v{version}");
        self.progress.set_message(message.clone());
        self.progress.inc(1);
        self.embedded.inc(Some(&message));
    }

    pub fn on_fetch_complete(&self) {
        self.progress.set_message("");
        self.progress.finish_and_clear();
        self.embedded.finish("Fetched latest versions");
    }
}
