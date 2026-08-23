#[cfg(feature = "python-managed")]
pub(crate) mod dir;
pub(crate) mod find;
#[cfg(feature = "python-managed")]
pub(crate) mod install;
pub(crate) mod list;
#[cfg(feature = "python-managed")]
pub(crate) mod pin;
#[cfg(feature = "python-managed")]
pub(crate) mod uninstall;
#[cfg(feature = "python-managed")]
pub(crate) mod update_shell;

#[cfg(feature = "python-managed")]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(super) enum ChangeEventKind {
    /// The Python version was uninstalled.
    Removed,
    /// The Python version was installed.
    Added,
    /// The Python version was reinstalled.
    Reinstalled,
}

#[cfg(feature = "python-managed")]
#[derive(Debug)]
pub(super) struct ChangeEvent {
    key: uv_python::PythonInstallationKey,
    kind: ChangeEventKind,
}
