//! Clean Language Manager Library
//!
//! This library provides the core functionality for the `cleen` and `frame` CLIs
//! and exposes a small programmatic API for other Clean Language components
//! (currently: the compiler's `cln update` command) to trigger installs without
//! shelling out.

pub mod commands;
pub mod core;
pub mod error;
pub mod plugin;
pub mod utils;

use crate::core::config::Config;
use crate::error::CleenError;
use std::path::PathBuf;

/// A version spec accepted by the `install` API. Accepts the same shapes as
/// the CLI: `"latest"`, `"0.30.109"`, or `"v0.30.109"`.
pub type VersionSpec<'a> = &'a str;

/// Result of a successful (or idempotent-successful) install.
#[derive(Debug, Clone)]
pub struct InstalledVersion {
    /// The resolved clean version (no `v` prefix), e.g. `"0.30.109"`.
    pub version: String,
    /// Absolute path to the installed `cln` binary.
    pub binary_path: PathBuf,
    /// `true` when this call installed the version; `false` when it was
    /// already present on disk and the call was a no-op.
    pub newly_installed: bool,
}

/// Install a Clean Language compiler version programmatically.
///
/// This is the library entry point intended for the compiler's future
/// `cln update` command. It:
///
/// - Resolves `"latest"` against GitHub if requested.
/// - Skips the interactive Frame CLI prompt (never touches stdin).
/// - Fires an install-kind heartbeat on success (see [`core::heartbeat`]).
/// - Returns `Ok(InstalledVersion { newly_installed: false, .. })` when the
///   version was already installed, so callers doing self-update loops can
///   treat the "already latest" case as success rather than an error.
///
/// Failures (network, GitHub 404, extraction errors, unusable binary) surface
/// as [`CleenError`].
pub fn install(spec: VersionSpec<'_>) -> Result<InstalledVersion, CleenError> {
    // Check whether the version is already present before delegating. The
    // existing CLI-facing `install_version` treats "already installed" as an
    // error via `VersionAlreadyInstalled`, which is the right behavior for a
    // human at the terminal but the wrong behavior for a self-update loop.
    // We short-circuit here so the caller gets a clean `Ok`.
    let config = Config::load()?;

    if let Some(resolved) = resolve_local_if_present(&config, spec) {
        let binary_path = config.get_version_binary(&resolved);
        if binary_path.exists() {
            // Still fire a heartbeat: from the dashboard's perspective the
            // version is active on this machine right now, which is exactly
            // what fix_released → fix_installed wants to hear.
            core::heartbeat::send_install();
            return Ok(InstalledVersion {
                version: resolved,
                binary_path,
                newly_installed: false,
            });
        }
    }

    // Delegate to the existing CLI-shared installer with prompts suppressed.
    commands::install::install_version(spec, /* with_frame */ false, /* no_frame */ true)?;

    // Reload config and resolve the installed binary. `install_version`
    // resolves "latest" internally against GitHub, so re-derive from what's
    // now on disk rather than trusting the input spec.
    let config = Config::load()?;
    let resolved = resolve_from_disk(&config, spec).ok_or_else(|| CleenError::VersionNotFound {
        version: spec.to_string(),
    })?;
    let binary_path = config.get_version_binary(&resolved);

    core::heartbeat::send_install();

    Ok(InstalledVersion {
        version: resolved,
        binary_path,
        newly_installed: true,
    })
}

/// Return the resolved clean version if a matching install already exists on
/// disk. For the literal spec `"latest"` this returns `None` — the check has
/// to hit GitHub to know what "latest" means today.
fn resolve_local_if_present(config: &Config, spec: &str) -> Option<String> {
    if spec.eq_ignore_ascii_case("latest") {
        return None;
    }
    let clean = core::version::normalize::to_clean_version(spec);
    if config.get_version_dir(&clean).exists() {
        Some(clean)
    } else {
        None
    }
}

/// After a successful `install_version`, find which version was actually
/// installed. For a pinned spec this is deterministic. For `"latest"`, we
/// pick the most recently modified version directory that also has the
/// expected binary present.
fn resolve_from_disk(config: &Config, spec: &str) -> Option<String> {
    if !spec.eq_ignore_ascii_case("latest") {
        let clean = core::version::normalize::to_clean_version(spec);
        return config.get_version_dir(&clean).exists().then_some(clean);
    }
    // "latest": scan versions dir and pick the newest by mtime.
    let versions_dir = config.get_versions_dir();
    let mut newest: Option<(std::time::SystemTime, String)> = None;
    let read = std::fs::read_dir(&versions_dir).ok()?;
    for entry in read.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if !meta.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        // Skip the frame subdir — it's not a compiler version.
        if name == "frame" {
            continue;
        }
        let mtime = meta.modified().unwrap_or(std::time::UNIX_EPOCH);
        match &newest {
            Some((prev, _)) if *prev >= mtime => {}
            _ => newest = Some((mtime, name)),
        }
    }
    newest.map(|(_, v)| v)
}
