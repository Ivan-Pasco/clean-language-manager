//! Tests for the eviction-graveyard prune path.
//!
//! Asserts:
//!   - `prune_graveyards` removes `*.locked-*` siblings under a directory
//!     and leaves non-graveyard entries alone.
//!   - `count_graveyards` matches the prune count for the same layout.
//!   - The graveyard summary in `commands/cleanup` walks both root-level
//!     and per-plugin version-level graveyards.

use cleen::commands::cleanup::{cleanup_graveyards_execute, graveyard_summary};
use cleen::core::config::Config;
use cleen::utils::fs as fs_utils;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

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

fn make_graveyard(parent: &Path, name: &str) {
    let path = parent.join(name);
    fs::create_dir_all(&path).unwrap();
    // Plant a small payload so size accounting has something to measure.
    fs::write(path.join("plugin.wasm"), b"\0asm\x01\0\0\0").unwrap();
}

#[test]
fn prune_graveyards_removes_locked_entries_only() {
    let tmp = TempDir::new().unwrap();
    let parent = tmp.path();

    make_graveyard(parent, "frame.ui.locked-1782142351333127000-47755");
    make_graveyard(parent, "frame.client.locked-1782188027018845000-73081");
    fs::create_dir_all(parent.join("frame.ui")).unwrap();
    fs::write(parent.join("config.json"), b"{}").unwrap();

    let (count, bytes) = fs_utils::prune_graveyards(parent);
    assert_eq!(count, 2);
    assert!(bytes > 0);

    assert!(
        !parent
            .join("frame.ui.locked-1782142351333127000-47755")
            .exists(),
        "root-level graveyard must be removed"
    );
    assert!(
        !parent
            .join("frame.client.locked-1782188027018845000-73081")
            .exists(),
        "root-level graveyard must be removed"
    );
    assert!(
        parent.join("frame.ui").exists(),
        "non-graveyard dir must be preserved"
    );
    assert!(
        parent.join("config.json").exists(),
        "non-graveyard file must be preserved"
    );
}

#[test]
fn count_graveyards_matches_actual_count() {
    let tmp = TempDir::new().unwrap();
    let parent = tmp.path();

    make_graveyard(parent, "a.locked-1");
    make_graveyard(parent, "b.locked-2");
    make_graveyard(parent, "c.locked-3");
    fs::create_dir_all(parent.join("a")).unwrap();

    assert_eq!(fs_utils::count_graveyards(parent), 3);
}

#[test]
fn count_graveyards_returns_zero_for_missing_dir() {
    let tmp = TempDir::new().unwrap();
    let missing = tmp.path().join("does-not-exist");
    assert_eq!(fs_utils::count_graveyards(&missing), 0);
}

#[test]
fn prune_graveyards_returns_zero_for_clean_dir() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("frame.ui")).unwrap();

    let (count, bytes) = fs_utils::prune_graveyards(tmp.path());
    assert_eq!(count, 0);
    assert_eq!(bytes, 0);
}

#[test]
fn graveyard_summary_walks_root_and_per_plugin_dirs() {
    let tmp = TempDir::new().unwrap();
    let cfg = test_config(tmp.path());
    let plugins_dir = cfg.get_plugins_dir();
    fs::create_dir_all(&plugins_dir).unwrap();

    // Root-level graveyard (from evict_locked_plugin_root).
    make_graveyard(&plugins_dir, "frame.ui.locked-1");
    // Per-plugin version graveyard (from evict_locked_dir).
    let plugin_dir = plugins_dir.join("frame.client");
    fs::create_dir_all(&plugin_dir).unwrap();
    make_graveyard(&plugin_dir, "1.2.3.locked-2");
    // Active version dir, no graveyard suffix — must NOT be counted.
    fs::create_dir_all(plugin_dir.join("1.2.4")).unwrap();
    fs::write(
        plugin_dir.join("1.2.4").join("plugin.wasm"),
        b"\0asm\x01\0\0\0",
    )
    .unwrap();

    let (count, bytes) = graveyard_summary(&cfg);
    assert_eq!(count, 2, "must include root-level + per-plugin graveyards");
    assert!(bytes > 0);
}

#[test]
fn cleanup_graveyards_execute_removes_both_layers() {
    let tmp = TempDir::new().unwrap();
    // cleanup_graveyards_execute calls Config::load() and reads
    // ~/.cleen/config.json — we can't substitute a temp config without
    // touching the user's real one. Use the helper layer instead.
    let plugins_dir = tmp.path().join("plugins");
    fs::create_dir_all(&plugins_dir).unwrap();

    make_graveyard(&plugins_dir, "frame.auth.locked-1");
    let plugin_dir = plugins_dir.join("frame.client");
    fs::create_dir_all(&plugin_dir).unwrap();
    make_graveyard(&plugin_dir, "1.2.3.locked-2");
    make_graveyard(&plugin_dir, "1.2.4.locked-3");

    // Mirror what cleanup_graveyards_execute does internally.
    let (root_count, _) = fs_utils::prune_graveyards(&plugins_dir);
    assert_eq!(root_count, 1);

    let mut per_plugin = 0;
    for entry in fs::read_dir(&plugins_dir).unwrap().flatten() {
        if entry.path().is_dir() {
            per_plugin += fs_utils::prune_graveyards(&entry.path()).0;
        }
    }
    assert_eq!(per_plugin, 2);

    // Calling the real execute path here would touch the user's
    // ~/.cleen, so we just confirm the call compiles and is reachable.
    let _ = cleanup_graveyards_execute;
}
