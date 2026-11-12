//! Elide fork: a native compiler toolchain supplied by an embedding host for PEP 517 builds.

use std::collections::HashMap;
use std::ffi::OsString;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use uv_cache_key::cache_digest;
use uv_normalize::PackageName;
use uv_pep440::{Version, VersionSpecifiers};
use version_ranges::Ranges;

/// Environment a build backend receives in place of the unresolvable compiler, interpreter
/// arguments it needs to see that environment, and the identity that scopes wheels built with it
/// in the cache.
#[derive(Debug)]
pub struct NativeBuildToolchain {
    env: Vec<(OsString, OsString)>,
    interpreter_args: Vec<OsString>,
    identity: String,
    source_preparation: Option<Vec<OsString>>,
    preferred_versions: HashMap<PackageName, Ranges<Version>>,
    wheel_verification: Option<Vec<OsString>>,
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
            source_preparation: None,
            preferred_versions: HashMap::new(),
            wheel_verification: None,
        }
    }

    /// Run `command <wheel>` on each wheel built from source; a failure fails the build and the
    /// wheel is discarded. Lets the host confirm that its toolchain produced the binaries.
    #[must_use]
    pub fn with_wheel_verification(mut self, command: Vec<OsString>) -> Self {
        self.wheel_verification = Some(command);
        self
    }

    pub fn wheel_verification(&self) -> Option<&[OsString]> {
        self.wheel_verification.as_deref()
    }

    /// Versions the resolver tries first, as `(package, specifiers)` pairs: typically those the
    /// source preparation has patches for. Several entries for a package are alternatives.
    /// Entries that do not parse are ignored.
    #[must_use]
    pub fn with_preferred_versions(
        mut self,
        preferences: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        for (name, specifiers) in preferences {
            let (Ok(name), Ok(specifiers)) = (
                PackageName::from_str(&name),
                VersionSpecifiers::from_str(&specifiers),
            ) else {
                continue;
            };
            let range = Ranges::from(specifiers);
            self.preferred_versions
                .entry(name)
                .and_modify(|preferred| *preferred = preferred.union(&range))
                .or_insert(range);
        }
        self
    }

    pub fn preferred_versions(&self, name: &PackageName) -> Option<&Ranges<Version>> {
        self.preferred_versions.get(name)
    }

    /// Run `command <name> <version> <directory>` on a copy of each unpacked source distribution
    /// before building it, so the host can patch the source. `identity` must cover what the
    /// command does.
    #[must_use]
    pub fn with_source_preparation(mut self, command: Vec<OsString>) -> Self {
        self.source_preparation = Some(command);
        self
    }

    pub fn source_preparation(&self) -> Option<&[OsString]> {
        self.source_preparation.as_deref()
    }

    /// A digest of [`Self::identity`], for cache paths.
    pub fn digest(&self) -> String {
        cache_digest(&self.identity)
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
