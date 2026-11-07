use indicatif::{MultiProgress, ProgressBar};

use uv_command_support::{
    Printer,
    embedded_progress::{self, ProgressUnit},
    progress::{Direction, ProgressReporter, progress_target},
};
use uv_pep440::Version;

pub(crate) struct BinaryDownloadReporter {
    reporter: ProgressReporter,
}

impl BinaryDownloadReporter {
    /// Initialize a [`BinaryDownloadReporter`] for a single binary download.
    pub(crate) fn single(printer: Printer) -> Self {
        let multi_progress = MultiProgress::with_draw_target(progress_target(printer));
        let root = multi_progress.add(ProgressBar::with_draw_target(
            None,
            progress_target(printer),
        ));
        let embedded_root =
            embedded_progress::start_task(None, "Downloading binary", None, ProgressUnit::Count);
        let reporter =
            ProgressReporter::new(root, multi_progress, printer).with_embedded_root(embedded_root);
        Self { reporter }
    }
}

impl uv_bin_install::Reporter for BinaryDownloadReporter {
    fn on_download_start(&self, name: &str, version: &Version, size: Option<u64>) -> usize {
        self.reporter
            .on_request_start(Direction::Download, format!("{name} v{version}"), size)
    }

    fn on_download_progress(&self, id: usize, inc: u64) {
        self.reporter.on_request_progress(id, inc);
    }

    fn on_download_complete(&self, id: usize) {
        self.reporter.on_request_complete(Direction::Download, id);
    }
}
