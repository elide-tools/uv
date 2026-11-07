#[cfg(feature = "auth")]
pub(crate) use auth::dir::dir as auth_dir;
#[cfg(feature = "auth")]
pub(crate) use auth::helper::helper as auth_helper;
#[cfg(feature = "auth")]
pub(crate) use auth::login::login as auth_login;
#[cfg(feature = "auth")]
pub(crate) use auth::logout::logout as auth_logout;
#[cfg(feature = "auth")]
pub(crate) use auth::token::token as auth_token;
#[cfg(feature = "cache")]
pub(crate) use cache_clean::cache_clean;
#[cfg(feature = "cache")]
pub(crate) use cache_dir::cache_dir;
#[cfg(feature = "cache")]
pub(crate) use cache_prune::cache_prune;
#[cfg(feature = "cache")]
pub(crate) use cache_size::cache_size;
pub(crate) use help::help;
#[cfg(feature = "pip")]
pub(crate) use pip::check::pip_check;
#[cfg(feature = "pip")]
pub(crate) use pip::compile::pip_compile;
#[cfg(feature = "pip")]
pub(crate) use pip::freeze::pip_freeze;
#[cfg(feature = "pip")]
pub(crate) use pip::install::pip_install;
#[cfg(feature = "pip")]
pub(crate) use pip::list::pip_list;
#[cfg(feature = "pip")]
pub(crate) use pip::show::pip_show;
#[cfg(feature = "pip")]
pub(crate) use pip::sync::pip_sync;
#[cfg(feature = "pip")]
pub(crate) use pip::tree::pip_tree;
#[cfg(feature = "pip")]
pub(crate) use pip::uninstall::pip_uninstall;
#[cfg(feature = "project")]
pub(crate) use project::add::add;
#[cfg(feature = "audit")]
pub(crate) use project::audit::audit;
#[cfg(feature = "project")]
pub(crate) use project::check::check;
#[cfg(feature = "project")]
pub(crate) use project::export::export;
#[cfg(feature = "project")]
pub(crate) use project::format::format;
#[cfg(feature = "init")]
pub(crate) use project::init::init;
#[cfg(feature = "project")]
pub(crate) use project::lock::lock;
#[cfg(feature = "project")]
pub(crate) use project::remove::remove;
#[cfg(feature = "run")]
pub(crate) use project::run::{ParsedRunCommand, RunCommand, run};
#[cfg(feature = "project")]
pub(crate) use project::sync::sync;
#[cfg(feature = "project")]
pub(crate) use project::tree::tree;
#[cfg(feature = "project")]
pub(crate) use project::upgrade::upgrade;
#[cfg(feature = "project")]
pub(crate) use project::version::project_version;
#[cfg(all(feature = "python", feature = "python-managed"))]
pub(crate) use python::dir::dir as python_dir;
#[cfg(feature = "python")]
pub(crate) use python::find::find as python_find;
#[cfg(feature = "python")]
pub(crate) use python::find::find_script as python_find_script;
#[cfg(all(feature = "python", feature = "python-managed"))]
pub(crate) use python::install::install as python_install;
#[cfg(feature = "python")]
pub(crate) use python::list::list as python_list;
#[cfg(all(feature = "python", feature = "python-managed"))]
pub(crate) use python::pin::pin as python_pin;
#[cfg(all(feature = "python", feature = "python-managed"))]
pub(crate) use python::uninstall::uninstall as python_uninstall;
#[cfg(all(feature = "python", feature = "python-managed"))]
pub(crate) use python::update_shell::update_shell as python_update_shell;
#[cfg(feature = "self-update")]
pub(crate) use self_update::self_update;
#[cfg(all(feature = "tool", feature = "audit"))]
pub(crate) use tool::audit::audit as tool_audit;
#[cfg(feature = "tool")]
pub(crate) use tool::dir::dir as tool_dir;
#[cfg(feature = "tool")]
pub(crate) use tool::install::install as tool_install;
#[cfg(feature = "tool")]
pub(crate) use tool::list::list as tool_list;
#[cfg(feature = "tool")]
pub(crate) use tool::run::run as tool_run;
#[cfg(feature = "tool")]
pub(crate) use tool::uninstall::uninstall as tool_uninstall;
#[cfg(feature = "tool")]
pub(crate) use tool::update_shell::update_shell as tool_update_shell;
#[cfg(feature = "tool")]
pub(crate) use tool::upgrade::upgrade as tool_upgrade;
#[cfg(feature = "build")]
pub(crate) use uv_build_commands::build_frontend;
/// The process status for a completed command, re-exported for embedding hosts.
pub use uv_command_support::ExitStatus;
#[cfg(feature = "cache")]
pub(crate) use uv_console::human_readable_bytes;
#[cfg(feature = "publish")]
pub(crate) use uv_publish_commands::publish;
#[cfg(feature = "venv")]
pub(crate) use venv::venv;
#[cfg(feature = "self-commands")]
pub(crate) use version::self_version;
#[cfg(feature = "workspace")]
pub(crate) use workspace::dir::dir;
#[cfg(feature = "workspace")]
pub(crate) use workspace::list::list;
#[cfg(feature = "workspace")]
pub(crate) use workspace::metadata::metadata;

#[cfg(feature = "auth")]
mod auth;
#[cfg(feature = "build")]
pub(crate) mod build_backend;
#[cfg(feature = "cache")]
mod cache_clean;
#[cfg(feature = "cache")]
mod cache_dir;
#[cfg(feature = "cache")]
mod cache_prune;
#[cfg(feature = "cache")]
mod cache_size;
pub(crate) mod diagnostics;
mod help;
#[cfg(feature = "pip")]
pub(crate) use uv_pip_commands as pip;
#[cfg(any(
    feature = "project",
    feature = "run",
    feature = "init",
    feature = "audit"
))]
pub(crate) use uv_project_commands as project;
#[cfg(feature = "python")]
pub(crate) use uv_python_commands as python;
#[cfg(feature = "cache")]
pub(crate) mod reporters;
#[cfg(feature = "self-update")]
mod self_update;
#[cfg(feature = "tool")]
pub(crate) use uv_tool_commands as tool;
#[cfg(feature = "venv")]
mod venv;
#[cfg(feature = "self-commands")]
mod version;
#[cfg(feature = "workspace")]
pub(crate) use uv_workspace_commands as workspace;

#[cfg(feature = "project")]
pub(crate) use uv_project_commands::ScriptPath;

#[cfg(all(test, feature = "project"))]
mod error_tests {
    use std::io::{Error, ErrorKind};

    use anyhow::bail;
    use insta::assert_snapshot;

    use uv_command_support::UvError;
    use uv_environment_operations::EnvironmentError;
    use uv_project_commands::ProjectError;
    use uv_resolve_operations::Error as ResolveError;

    #[test]
    fn resolution_context_missing_requirements() -> anyhow::Result<()> {
        let error = ResolveError::Requirements(uv_requirements::Error::Io(Error::new(
            ErrorKind::NotFound,
            "requirements failure",
        )));

        // Tool requirements replace the script context.
        let error = error
            .with_resolution_context("script")
            .with_resolution_context("tool");

        let UvError::User(error) = UvError::from(error) else {
            bail!("expected a user error");
        };
        assert_snapshot!(format!("{error:#}"), @"Failed to resolve tool requirement: requirements failure");

        Ok(())
    }

    #[test]
    fn resolution_context_unreadable_requirements() -> anyhow::Result<()> {
        let error = ResolveError::Requirements(uv_requirements::Error::Io(Error::new(
            ErrorKind::PermissionDenied,
            "requirements failure",
        )));

        // Tool requirements replace the script context.
        let error = error
            .with_resolution_context("script")
            .with_resolution_context("tool");

        let UvError::Unexpected(error) = UvError::from(error) else {
            bail!("expected an unexpected error");
        };
        assert_snapshot!(format!("{error:#}"), @"Failed to resolve tool requirement: requirements failure");

        Ok(())
    }

    #[test]
    fn resolution_context_leaves_other_errors_unchanged() -> anyhow::Result<()> {
        let error = ResolveError::Io(Error::new(
            ErrorKind::PermissionDenied,
            "cache write failed",
        ));

        // Resolution context only applies to requirements and solver errors.
        let error = error.with_resolution_context("tool");

        let UvError::Unexpected(error) = UvError::from(error) else {
            bail!("expected an unexpected error");
        };
        assert_snapshot!(format!("{error:#}"), @"cache write failed");

        Ok(())
    }

    #[test]
    fn project_requirements_use_operation_classification() -> anyhow::Result<()> {
        let error = EnvironmentError::Requirements(uv_requirements::Error::Io(Error::new(
            ErrorKind::NotFound,
            "requirements failure",
        )));

        // A project wrapper retains the requirements error's classification.
        let UvError::User(_) = UvError::from(ProjectError::from(error)) else {
            bail!("expected a user error");
        };

        Ok(())
    }
}
