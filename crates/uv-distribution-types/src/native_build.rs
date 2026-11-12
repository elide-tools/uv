//! Elide fork: a native compiler toolchain supplied by an embedding host for PEP 517 builds.

use std::ffi::OsString;
use std::sync::{Arc, RwLock};

/// Environment a build backend receives in place of the unresolvable compiler, interpreter
/// arguments it needs to see that environment, and the identity that scopes wheels built with it
/// in the cache.
#[derive(Debug)]
pub struct NativeBuildToolchain {
    env: Vec<(OsString, OsString)>,
    interpreter_args: Vec<OsString>,
    identity: String,
}

impl NativeBuildToolchain {
    /// `identity` must change whenever the produced binaries could (compiler, target, SDK, flags).
    pub fn new(
        env: Vec<(OsString, OsString)>,
        interpreter_args: Vec<OsString>,
        identity: String,
    ) -> Self {
        Self {
            env,
            interpreter_args,
            identity,
        }
    }

    pub fn env(&self) -> &[(OsString, OsString)] {
        &self.env
    }

    /// Passed to the build environment's interpreter before `-c`.
    pub fn interpreter_args(&self) -> &[OsString] {
        &self.interpreter_args
    }

    pub fn identity(&self) -> &str {
        &self.identity
    }
}

#[derive(Debug, Clone)]
enum Policy {
    Toolchain(Arc<NativeBuildToolchain>),
    Refuse,
}

static ACTIVE: RwLock<Option<Policy>> = RwLock::new(None);

/// Restores the previous policy on drop.
#[must_use]
pub struct NativeBuildPolicyGuard {
    previous: Option<Policy>,
}

impl Drop for NativeBuildPolicyGuard {
    fn drop(&mut self) {
        if let Ok(mut active) = ACTIVE.write() {
            *active = self.previous.take();
        }
    }
}

fn install(policy: Policy) -> NativeBuildPolicyGuard {
    let previous = ACTIVE
        .write()
        .ok()
        .and_then(|mut active| active.replace(policy));
    NativeBuildPolicyGuard { previous }
}

/// Build backends get `toolchain` in place of the unresolvable compiler.
pub fn install_native_build_toolchain(toolchain: NativeBuildToolchain) -> NativeBuildPolicyGuard {
    install(Policy::Toolchain(Arc::new(toolchain)))
}

/// No PEP 517 build runs, editables included, as when uv is built without `source-build`.
pub fn refuse_source_builds() -> NativeBuildPolicyGuard {
    install(Policy::Refuse)
}

pub fn native_build_toolchain() -> Option<Arc<NativeBuildToolchain>> {
    match ACTIVE.read().ok()?.as_ref()? {
        Policy::Toolchain(toolchain) => Some(toolchain.clone()),
        Policy::Refuse => None,
    }
}

pub fn source_builds_refused() -> bool {
    ACTIVE
        .read()
        .is_ok_and(|active| matches!(*active, Some(Policy::Refuse)))
}
