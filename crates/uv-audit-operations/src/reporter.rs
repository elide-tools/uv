use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use uv_command_support::Printer;
use uv_command_support::embedded_progress::{EmbeddedTask, ProgressUnit};
use uv_command_support::progress::progress_target;

#[derive(Debug)]
pub(crate) struct AuditReporter {
    progress: ProgressBar,
    embedded: EmbeddedTask,
}

impl From<Printer> for AuditReporter {
    fn from(printer: Printer) -> Self {
        let progress = ProgressBar::with_draw_target(None, progress_target(printer));
        progress.enable_steady_tick(Duration::from_millis(200));
        progress.set_style(
            ProgressStyle::with_template("{spinner:.white} {wide_msg:.dim}")
                .unwrap()
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
        );
        progress.set_message("Auditing dependencies...");
        let embedded = EmbeddedTask::start("Auditing dependencies", None, ProgressUnit::None);
        Self { progress, embedded }
    }
}

impl AuditReporter {
    pub(crate) fn on_audit_complete(&self) {
        self.progress.set_message("");
        self.progress.finish_and_clear();
        self.embedded.finish("Audited dependencies");
    }
}
