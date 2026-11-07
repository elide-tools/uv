//! Commands for discovering and managing Python installations.

#[cfg(feature = "python-managed")]
pub mod dir;
pub mod find;
#[cfg(feature = "python-managed")]
pub mod install;
pub mod list;
#[cfg(feature = "python-managed")]
pub mod pin;
#[cfg(feature = "python-managed")]
pub mod uninstall;
#[cfg(feature = "python-managed")]
pub mod update_shell;

#[cfg(feature = "python-managed")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum ChangeEventKind {
    /// The Python version was uninstalled.
    Removed,
    /// The Python version was installed.
    Added,
    /// The Python version was reinstalled.
    Reinstalled,
}

#[cfg(feature = "python-managed")]
#[derive(Debug)]
pub(crate) struct ChangeEvent {
    key: uv_python_types::PythonInstallationKey,
    kind: ChangeEventKind,
}
