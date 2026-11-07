use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;
use uv_cache::Removal;
use uv_command_support::Printer;
use uv_command_support::embedded_progress::{EmbeddedTask, ProgressUnit};
use uv_command_support::progress::progress_target;

#[derive(Debug)]
pub(crate) struct CleaningDirectoryReporter {
    bar: ProgressBar,
    embedded: EmbeddedTask,
}

impl CleaningDirectoryReporter {
    /// Initialize a [`CleaningDirectoryReporter`] for cleaning the cache directory.
    pub(crate) fn new(printer: Printer, max: Option<usize>) -> Self {
        let bar = ProgressBar::with_draw_target(max.map(|m| m as u64), progress_target(printer));
        bar.set_style(
            ProgressStyle::with_template("{prefix} [{bar:20}] {percent}%")
                .unwrap()
                .progress_chars("=> "),
        );
        bar.set_prefix(format!("{}", "Cleaning".bold().cyan()));
        let embedded = EmbeddedTask::start(
            "Cleaning cache directories",
            max.map(|m| m as u64),
            ProgressUnit::Count,
        );
        Self { bar, embedded }
    }
}

impl uv_cache::CleanReporter for CleaningDirectoryReporter {
    fn on_clean(&self) {
        self.bar.inc(1);
        self.embedded.inc(None);
    }

    fn on_complete(&self) {
        self.bar.finish_and_clear();
        self.embedded.finish("Cleaned cache directories");
    }
}

#[derive(Debug)]
pub(crate) struct CleaningPackageReporter {
    bar: ProgressBar,
    embedded: EmbeddedTask,
}

impl CleaningPackageReporter {
    /// Initialize a [`CleaningPackageReporter`] for cleaning packages from the cache.
    pub(crate) fn new(printer: Printer, max: Option<usize>) -> Self {
        let bar = ProgressBar::with_draw_target(max.map(|m| m as u64), progress_target(printer));
        bar.set_style(
            ProgressStyle::with_template("{prefix} [{bar:20}] {pos}/{len}{msg}")
                .unwrap()
                .progress_chars("=> "),
        );
        bar.set_prefix(format!("{}", "Cleaning".bold().cyan()));
        let embedded = EmbeddedTask::start(
            "Cleaning cached packages",
            max.map(|m| m as u64),
            ProgressUnit::Count,
        );
        Self { bar, embedded }
    }

    pub(crate) fn on_clean(&self, package: &str, removal: &Removal) {
        self.bar.inc(1);
        let message = format!(
            ": {}, {} files {} folders removed",
            package, removal.num_files, removal.num_dirs,
        );
        self.bar.set_message(message.clone());
        self.embedded.inc(Some(&message));
    }

    pub(crate) fn on_complete(&self) {
        self.bar.finish_and_clear();
        self.embedded.finish("Cleaned cached packages");
    }
}
