# Elide fork of uv

- **Upstream:** https://github.com/astral-sh/uv
- **Upstream commit at fork:** d46a7b67b0dacdc4d8b989fa66fff90ae06e47fd
- **Upstream commit date:** 2026-04-23 16:17:32 +0000
- **Forked on:** 2026-04-23
- **Fork purpose:** embedded as `pypi` resolver in WHIPLASH (`crates/resolvers`).

## Elide-applied patches

In reverse chronological order:

- (2026-04-23) — `chore(uv-keyring): drop native-auth default features`
  - `crates/uv-keyring/Cargo.toml`: changed `default = ["apple-native", "secret-service", "windows-native"]` to `default = []`. The native-auth integrations (macOS Keychain, Linux secret-service, Windows Credential Store) are now opt-in rather than on-by-default. Embedded consumers that don't use `uv publish` credential storage via the OS keyring get a smaller default build.
  - Note: does NOT drop `security-framework` from the graph — that crate is pulled independently via `rustls-native-certs` for system-cert loading, unrelated to the keyring.
  - Attempted parallel cut: removing `"schemars"` activations from uv-cli/uv-configuration/uv-distribution-types/uv-settings. REVERTED because `uv-settings` and `uv-distribution-types` have unconditional `schemars::JsonSchema` derives in source that can't be cleanly feature-gated without ~60+ source edits per crate. Deferred as a bigger fork patch.

- (2026-04-23) — `feat: gate global tracing-subscriber install`
  - `crates/uv/Cargo.toml`: added feature `install-tracing-subscriber` to `[features]`, default-on. Threaded into `default = [...]` alongside existing defaults.
  - `crates/uv/src/logging.rs`: `setup_logging` now has two impls — the real one gated by `#[cfg(feature = "install-tracing-subscriber")]`, and a stub that returns `Ok(())` when the feature is off. The stub lets tracing events continue to emit through the macros without installing a global subscriber, so the embedder can own the subscriber and receive uv events alongside other emitters (orogene, Elide, …).

- (2026-04-23) — `fix(uv-extract): disambiguate .compat() to FuturesAsyncReadCompatExt`
  - `crates/uv-extract/src/stream.rs:196`: replaced `entry.reader_mut().compat()` with `FuturesAsyncReadCompatExt::compat(entry.reader_mut())`.
  - Reason: under WHIPLASH's toolchain (nightly-2026-04-15), both `futures::AsyncReadExt::compat` and `tokio_util::compat::FuturesAsyncReadCompatExt::compat` are in scope at the call site, yielding `E0034: multiple applicable items in scope`. uv's pinned stable `1.94.1` accepted the ambiguous call; newer strictness rejects it. Explicit trait-path disambiguation resolves without changing behavior (the intent is futures-AsyncRead → tokio-AsyncRead, which is the `FuturesAsyncReadCompatExt` direction).

## Known constraints

- `uv::main()` constructs its own tokio `Runtime` and must NOT be called from inside another tokio runtime. WHIPLASH works around this by running `pypi::run` on a non-runtime thread.
- Eventual programmatic API (library-level sync/install calls) will require forking out this constraint.

## Entrypoint signature

As of d46a7b67b0dacdc4d8b989fa66fff90ae06e47fd, uv exposes:

```rust
pub unsafe fn main<I, T>(args: I) -> ExitCode
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
```

Defined in `crates/uv/src/lib.rs` at line 2725. The function is generic over any iterator of `OsString`-convertible items. The `unsafe` requirement is documented as: "It is only safe to call this routine when it is known that multiple threads are not running" (due to `std::env::set_var` on startup). WHIPLASH must call this from a dedicated non-runtime thread and must ensure no other threads are active at the point of the call.

There is no `manage_python_downloads` boolean argument at this fork point — the signature is purely `(args: I) -> ExitCode`. Python download management is controlled via CLI flags in `args` instead.

## Known build issues

None. Standalone build (`cargo build -p uv`) verified clean on 2026-04-23 using rustc 1.94.1 (as pinned in this fork's `rust-toolchain.toml`). Cold build completed in approximately 1m 37s. WHIPLASH pins `nightly-2026-04-15`, which is newer than `1.94.1`; no toolchain pin relaxation was required.

## Sync procedure

```bash
git fetch upstream
git merge upstream/main
```
