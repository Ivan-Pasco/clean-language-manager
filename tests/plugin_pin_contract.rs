//! Contract tests for plugin pin resolution.
//!
//! Asserts the behaviors documented in
//! `foundation/platform-architecture/HOST_BRIDGE.md` "Plugin Pin Resolution":
//!   - `.active-version` is the single source of truth.
//!   - A ghost pin (marker present, plugin.wasm missing) resolves to `None`.
//!   - `cleen cleanup --plugins` never deletes the only on-disk version.
//!   - `activate_plugin_version_root` refuses to write a pin when the target
//!     `plugin.wasm` is missing.
//!   - A legacy `config.json` carrying the removed `active_plugins` map still
//!     deserializes and is saved without that field.

use cleen::commands::cleanup::{
    cleanup_plugins_with_config, plugin_cleanup_summary,
};
use cleen::core::config::{read_active_version, Config};
use cleen::plugin::activate_plugin_version_root;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Build a `Config` rooted at a temp directory so tests do not touch
/// `~/.cleen`. The struct fields are `pub`, so direct construction is
/// the supported path for tests.
fn test_config(cleen_dir: &Path) -> Config {
    Config {
        active_version: None,
        frame_version: None,
        server_version: None,
        cleen_dir: cleen_dir.to_path_buf(),
        auto_cleanup: false,
        github_api_token: None,
        check_updates: false,
        auto_offer_frame: false,
        last_update_check: None,
        last_self_update_check: None,
    }
}

/// Create a plugin version directory with `plugin.toml` and `plugin.wasm`
/// so it looks like an actual installed plugin.
fn install_plugin_version(plugins_dir: &Path, name: &str, version: &str) -> PathBuf {
    let version_dir = plugins_dir.join(name).join(version);
    fs::create_dir_all(&version_dir).unwrap();
    fs::write(
        version_dir.join("plugin.toml"),
        format!(
            "[plugin]\nname = \"{name}\"\nversion = \"{version}\"\n\n[compatibility]\nmin_compiler_version = \"0.0.0\"\n"
        ),
    )
    .unwrap();
    fs::write(version_dir.join("plugin.wasm"), b"\0asm\x01\0\0\0").unwrap();
    version_dir
}

/// Create a version directory but deliberately omit `plugin.wasm` so the
/// pin resolution helper treats it as a ghost.
fn install_plugin_version_without_wasm(plugins_dir: &Path, name: &str, version: &str) -> PathBuf {
    let version_dir = plugins_dir.join(name).join(version);
    fs::create_dir_all(&version_dir).unwrap();
    fs::write(
        version_dir.join("plugin.toml"),
        format!("[plugin]\nname = \"{name}\"\nversion = \"{version}\"\n"),
    )
    .unwrap();
    version_dir
}

fn write_active_version(plugins_dir: &Path, name: &str, version: &str) {
    let plugin_dir = plugins_dir.join(name);
    fs::create_dir_all(&plugin_dir).unwrap();
    fs::write(plugin_dir.join(".active-version"), version).unwrap();
}

#[test]
fn read_active_version_returns_none_for_ghost_pin_missing_dir() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();
    fs::create_dir_all(plugins_dir.join("frame.client")).unwrap();
    install_plugin_version(&plugins_dir, "frame.client", "1.2.2");

    // Pin points at a version that was never installed.
    write_active_version(&plugins_dir, "frame.client", "1.2.3");

    assert_eq!(read_active_version(&cfg, "frame.client"), None);
}

#[test]
fn read_active_version_returns_none_for_ghost_pin_missing_wasm() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();
    install_plugin_version_without_wasm(&plugins_dir, "frame.client", "1.2.3");
    write_active_version(&plugins_dir, "frame.client", "1.2.3");

    assert_eq!(read_active_version(&cfg, "frame.client"), None);
}

#[test]
fn read_active_version_returns_pinned_version_when_valid() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();
    install_plugin_version(&plugins_dir, "frame.client", "1.2.2");
    install_plugin_version(&plugins_dir, "frame.client", "1.2.3");
    write_active_version(&plugins_dir, "frame.client", "1.2.2");

    assert_eq!(
        read_active_version(&cfg, "frame.client"),
        Some("1.2.2".to_string())
    );
}

#[test]
fn read_active_version_returns_none_when_marker_missing() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();
    install_plugin_version(&plugins_dir, "frame.client", "2.0.0");

    // No `.active-version` marker — manager returns None and lets the
    // compiler's semver-max fallback take over.
    assert_eq!(read_active_version(&cfg, "frame.client"), None);
}

#[test]
fn read_active_version_returns_none_for_empty_marker() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();
    install_plugin_version(&plugins_dir, "frame.client", "1.0.0");
    write_active_version(&plugins_dir, "frame.client", "   \n  ");

    assert_eq!(read_active_version(&cfg, "frame.client"), None);
}

#[test]
fn activate_plugin_refuses_to_pin_missing_wasm() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();
    install_plugin_version_without_wasm(&plugins_dir, "frame.client", "1.2.3");

    let err = activate_plugin_version_root(&cfg, "frame.client", "1.2.3");
    assert!(err.is_err(), "expected ghost-pin guard to reject activation");

    // No marker should have been written, leaving the lookup at None.
    assert_eq!(read_active_version(&cfg, "frame.client"), None);
}

#[test]
fn activate_plugin_writes_marker_when_wasm_present() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();
    install_plugin_version(&plugins_dir, "frame.client", "1.2.3");

    activate_plugin_version_root(&cfg, "frame.client", "1.2.3").unwrap();
    assert_eq!(
        read_active_version(&cfg, "frame.client"),
        Some("1.2.3".to_string())
    );
}

#[test]
fn cleanup_plugins_never_deletes_only_on_disk_version() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();
    install_plugin_version(&plugins_dir, "frame.client", "1.2.2");
    // Ghost pin points at a non-existent version, leaving the lone
    // installed version visible as "not active". The safety guard must
    // refuse to delete it.
    write_active_version(&plugins_dir, "frame.client", "1.2.3");

    cleanup_plugins_with_config(&cfg).unwrap();

    assert!(
        plugins_dir.join("frame.client").join("1.2.2").exists(),
        "cleanup must not delete the only on-disk version of a plugin"
    );
}

#[test]
fn cleanup_plugins_removes_inactive_versions_when_multiple_exist() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();
    install_plugin_version(&plugins_dir, "frame.client", "1.0.0");
    install_plugin_version(&plugins_dir, "frame.client", "2.0.0");
    activate_plugin_version_root(&cfg, "frame.client", "2.0.0").unwrap();

    cleanup_plugins_with_config(&cfg).unwrap();

    assert!(plugins_dir.join("frame.client").join("2.0.0").exists());
    assert!(
        !plugins_dir.join("frame.client").join("1.0.0").exists(),
        "inactive version should be cleaned up"
    );
}

#[test]
fn plugin_cleanup_summary_returns_none_when_nothing_safe_to_remove() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();

    // Lone version of frame.client: protected by the single-version guard,
    // contributes nothing to the summary even with a stale pin.
    install_plugin_version(&plugins_dir, "frame.client", "1.2.2");
    write_active_version(&plugins_dir, "frame.client", "1.2.3");

    assert_eq!(plugin_cleanup_summary(&cfg), None);
}

#[test]
fn plugin_cleanup_summary_reports_inactive_versions_only() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();

    install_plugin_version(&plugins_dir, "frame.client", "1.0.0");
    install_plugin_version(&plugins_dir, "frame.client", "2.0.0");
    install_plugin_version(&plugins_dir, "frame.ui", "0.9.0");
    install_plugin_version(&plugins_dir, "frame.ui", "1.0.0");
    activate_plugin_version_root(&cfg, "frame.client", "2.0.0").unwrap();
    activate_plugin_version_root(&cfg, "frame.ui", "1.0.0").unwrap();

    let (count, bytes) = plugin_cleanup_summary(&cfg).expect("two inactive versions exist");
    assert_eq!(count, 2);
    assert!(bytes > 0);
}

#[test]
fn legacy_config_with_active_plugins_field_deserializes_and_drops_field_on_save() {
    let legacy_json = r#"{
        "active_version": "0.30.352",
        "cleen_dir": "/tmp/legacy-cleen",
        "auto_cleanup": false,
        "github_api_token": null,
        "last_update_check": null,
        "last_self_update_check": null,
        "active_plugins": {
            "frame.client": "1.2.3",
            "frame.ui": "2.0.0"
        }
    }"#;

    let cfg: Config = serde_json::from_str(legacy_json).expect("legacy config must deserialize");
    assert_eq!(cfg.active_version.as_deref(), Some("0.30.352"));

    let round_tripped = serde_json::to_string(&cfg).unwrap();
    assert!(
        !round_tripped.contains("active_plugins"),
        "active_plugins field must be dropped on save, got: {round_tripped}"
    );
}
