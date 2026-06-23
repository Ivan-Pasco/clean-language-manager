use crate::core::config::{read_active_version, Config};
use crate::error::Result;
use std::fs;
use std::path::PathBuf;

/// Information about a version that can be cleaned up
#[derive(Debug)]
pub struct CleanupCandidate {
    version: String,
    size_bytes: u64,
    is_active: bool,
    is_frame_dependency: bool,
}

/// List versions that can be cleaned up
pub fn list_cleanup_candidates(config: &Config) -> Result<Vec<CleanupCandidate>> {
    let versions_dir = config.get_versions_dir();
    let mut candidates = Vec::new();

    if !versions_dir.exists() {
        return Ok(candidates);
    }

    for entry in fs::read_dir(&versions_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let version = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // Skip the "frame" directory (it's for Frame CLI versions)
        if version == "frame" {
            continue;
        }

        // Calculate directory size
        let size_bytes = calculate_dir_size(&path).unwrap_or(0);

        // Check if this is the active version
        let is_active = config.active_version.as_ref() == Some(&version);

        // Check if Frame CLI depends on this version
        let is_frame_dependency = check_frame_dependency(config, &version);

        candidates.push(CleanupCandidate {
            version,
            size_bytes,
            is_active,
            is_frame_dependency,
        });
    }

    // Sort by version (oldest first based on semantic version parsing)
    candidates.sort_by(|a, b| compare_versions(&a.version, &b.version));

    Ok(candidates)
}

/// Compare two version strings semantically
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let parse_version = |v: &str| -> Vec<u32> {
        v.trim_start_matches('v')
            .split(['.', '-'])
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };

    let a_parts = parse_version(a);
    let b_parts = parse_version(b);

    for i in 0..std::cmp::max(a_parts.len(), b_parts.len()) {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);

        match a_val.cmp(&b_val) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    std::cmp::Ordering::Equal
}

/// Calculate total size of a directory
fn calculate_dir_size(path: &std::path::Path) -> Result<u64> {
    let mut total = 0;

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;

        if metadata.is_file() {
            total += metadata.len();
        } else if metadata.is_dir() {
            total += calculate_dir_size(&entry.path())?;
        }
    }

    Ok(total)
}

/// Check if Frame CLI depends on a compiler version
fn check_frame_dependency(config: &Config, version: &str) -> bool {
    // Only the active compiler version is a Frame dependency
    // Frame CLI needs the currently active compiler to work
    if config.frame_version.is_none() {
        return false;
    }

    // Only protect the active version for Frame
    config.active_version.as_ref() == Some(&version.to_string())
}

/// Count and total-size of compiler versions that `cleen cleanup` would
/// consider removable: every installed version that is neither the active
/// version nor the one the active Frame CLI depends on. Returns `None`
/// when nothing is removable so callers can skip the post-install hint
/// entirely rather than printing a noisy "0 versions to clean" line.
pub fn compiler_cleanup_summary(config: &Config) -> Option<(usize, u64)> {
    let candidates = list_cleanup_candidates(config).ok()?;
    let removable: Vec<_> = candidates
        .into_iter()
        .filter(|c| !c.is_active && !c.is_frame_dependency)
        .collect();
    if removable.is_empty() {
        return None;
    }
    let total: u64 = removable.iter().map(|c| c.size_bytes).sum();
    Some((removable.len(), total))
}

/// Same shape as [`compiler_cleanup_summary`] but for plugin versions.
/// Honors the single-version safety guard from `cleanup_plugins_execute`:
/// a plugin with only one version on disk contributes nothing, even if
/// its pin is stale (see HOST_BRIDGE.md "Plugin Pin Resolution").
pub fn plugin_cleanup_summary(config: &Config) -> Option<(usize, u64)> {
    let plugins_dir = config.get_plugins_dir();
    if !plugins_dir.exists() {
        return None;
    }

    let mut count = 0usize;
    let mut total = 0u64;

    let entries = fs::read_dir(&plugins_dir).ok()?;
    for plugin_entry in entries.flatten() {
        let plugin_path = plugin_entry.path();
        if !plugin_path.is_dir() {
            continue;
        }

        let plugin_name = plugin_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let active_version = read_active_version(config, &plugin_name);

        let mut version_dirs: Vec<(String, PathBuf)> = Vec::new();
        if let Ok(version_entries) = fs::read_dir(&plugin_path) {
            for version_entry in version_entries.flatten() {
                let version_path = version_entry.path();
                if !version_path.is_dir() {
                    continue;
                }
                let version = version_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                version_dirs.push((version, version_path));
            }
        }

        // Single (or zero) on-disk version: nothing safe to remove here.
        if version_dirs.len() <= 1 {
            continue;
        }

        for (version, version_path) in &version_dirs {
            if active_version.as_ref() == Some(version) {
                continue;
            }
            count += 1;
            total += calculate_dir_size(version_path).unwrap_or(0);
        }
    }

    if count == 0 {
        None
    } else {
        Some((count, total))
    }
}

/// Format bytes as human-readable size
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Run cleanup in dry-run mode (just show what would be removed)
pub fn cleanup_dry_run(keep_count: usize) -> Result<()> {
    let config = Config::load()?;
    let candidates = list_cleanup_candidates(&config)?;

    if candidates.is_empty() {
        println!("No compiler versions installed.");
        return Ok(());
    }

    // Separate protected and removable versions
    let (protected, removable): (Vec<_>, Vec<_>) = candidates
        .into_iter()
        .partition(|c| c.is_active || c.is_frame_dependency);

    // Keep the most recent N versions from removable
    let to_keep = if removable.len() > keep_count {
        &removable[removable.len() - keep_count..]
    } else {
        &removable[..]
    };

    let to_remove: Vec<_> = removable
        .iter()
        .filter(|c| !to_keep.iter().any(|k| k.version == c.version))
        .collect();

    println!("Cleanup analysis:");
    println!();

    // Show protected versions
    if !protected.is_empty() {
        println!("Protected versions (will NOT be removed):");
        for c in &protected {
            let reasons: Vec<&str> = [
                if c.is_active { Some("active") } else { None },
                if c.is_frame_dependency {
                    Some("frame dependency")
                } else {
                    None
                },
            ]
            .into_iter()
            .flatten()
            .collect();

            println!(
                "  {} ({}) - {}",
                c.version,
                format_size(c.size_bytes),
                reasons.join(", ")
            );
        }
        println!();
    }

    // Show versions to keep
    if !to_keep.is_empty() {
        println!("Versions to keep (most recent {}):", keep_count);
        for c in to_keep {
            println!("  {} ({})", c.version, format_size(c.size_bytes));
        }
        println!();
    }

    // Show versions to remove
    if to_remove.is_empty() {
        println!("No versions to remove.");
    } else {
        let total_size: u64 = to_remove.iter().map(|c| c.size_bytes).sum();
        println!("Versions to remove ({} total):", format_size(total_size));
        for c in &to_remove {
            println!("  {} ({})", c.version, format_size(c.size_bytes));
        }
        println!();
        println!("Run 'cleen cleanup --confirm' to remove these versions.");
    }

    Ok(())
}

/// Run cleanup and actually remove old versions
pub fn cleanup_execute(keep_count: usize) -> Result<()> {
    let config = Config::load()?;
    let candidates = list_cleanup_candidates(&config)?;

    if candidates.is_empty() {
        println!("No compiler versions installed.");
        return Ok(());
    }

    // Separate protected and removable versions
    let (protected, removable): (Vec<_>, Vec<_>) = candidates
        .into_iter()
        .partition(|c| c.is_active || c.is_frame_dependency);

    // Keep the most recent N versions from removable
    let to_keep_versions: Vec<String> = if removable.len() > keep_count {
        removable[removable.len() - keep_count..]
            .iter()
            .map(|c| c.version.clone())
            .collect()
    } else {
        removable.iter().map(|c| c.version.clone()).collect()
    };

    let to_remove: Vec<_> = removable
        .iter()
        .filter(|c| !to_keep_versions.contains(&c.version))
        .collect();

    if to_remove.is_empty() {
        println!("No versions to remove.");
        println!(
            "Keeping {} version(s) plus {} protected version(s).",
            to_keep_versions.len(),
            protected.len()
        );
        return Ok(());
    }

    let total_size: u64 = to_remove.iter().map(|c| c.size_bytes).sum();
    println!(
        "Removing {} version(s) to free {}...",
        to_remove.len(),
        format_size(total_size)
    );
    println!();

    let mut removed_count = 0;
    let mut freed_bytes = 0u64;

    for candidate in &to_remove {
        let version_dir = config.get_version_dir(&candidate.version);

        print!("  Removing {}... ", candidate.version);

        match fs::remove_dir_all(&version_dir) {
            Ok(()) => {
                println!("done ({})", format_size(candidate.size_bytes));
                removed_count += 1;
                freed_bytes += candidate.size_bytes;
            }
            Err(e) => {
                println!("failed: {}", e);
            }
        }
    }

    println!();
    println!(
        "Cleanup complete: removed {} version(s), freed {}",
        removed_count,
        format_size(freed_bytes)
    );

    Ok(())
}

/// Clean up old plugin versions
pub fn cleanup_plugins_dry_run() -> Result<()> {
    let config = Config::load()?;
    let plugins_dir = config.get_plugins_dir();

    if !plugins_dir.exists() {
        println!("No plugins installed.");
        return Ok(());
    }

    println!("Plugin cleanup analysis:");
    println!();

    let mut total_removable = 0u64;
    let mut found_any = false;

    for plugin_entry in fs::read_dir(&plugins_dir)? {
        let plugin_entry = plugin_entry?;
        let plugin_path = plugin_entry.path();

        if !plugin_path.is_dir() {
            continue;
        }

        let plugin_name = plugin_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let active_version = read_active_version(&config, &plugin_name);

        let mut versions: Vec<(String, u64)> = Vec::new();

        for version_entry in fs::read_dir(&plugin_path)? {
            let version_entry = version_entry?;
            let version_path = version_entry.path();

            if !version_path.is_dir() {
                continue;
            }

            let version = version_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            let size = calculate_dir_size(&version_path).unwrap_or(0);
            versions.push((version, size));
        }

        if versions.len() > 1 {
            found_any = true;
            println!("  {}:", plugin_name);

            for (version, size) in &versions {
                let is_active = active_version.as_ref() == Some(version);
                if is_active {
                    println!("    {} ({}) - active, keeping", version, format_size(*size));
                } else {
                    println!("    {} ({}) - can be removed", version, format_size(*size));
                    total_removable += size;
                }
            }
            println!();
        }
    }

    if !found_any {
        println!("No plugins with multiple versions found.");
    } else {
        println!("Total removable: {}", format_size(total_removable));
        println!();
        println!("Run 'cleen cleanup --plugins --confirm' to remove inactive plugin versions.");
    }

    Ok(())
}

/// Clean up inactive plugin versions
pub fn cleanup_plugins_execute() -> Result<()> {
    let config = Config::load()?;
    cleanup_plugins_with_config(&config)
}

/// Same as `cleanup_plugins_execute`, but accepts an explicit config so
/// integration tests can drive the safety-guard logic against a temp
/// `~/.cleen` layout without touching the user's real directory.
pub fn cleanup_plugins_with_config(config: &Config) -> Result<()> {
    let plugins_dir = config.get_plugins_dir();

    if !plugins_dir.exists() {
        println!("No plugins installed.");
        return Ok(());
    }

    println!("Cleaning up inactive plugin versions...");
    println!();

    let mut removed_count = 0;
    let mut freed_bytes = 0u64;

    for plugin_entry in fs::read_dir(&plugins_dir)? {
        let plugin_entry = plugin_entry?;
        let plugin_path = plugin_entry.path();

        if !plugin_path.is_dir() {
            continue;
        }

        let plugin_name = plugin_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let active_version = read_active_version(config, &plugin_name);

        // Collect candidate version directories first so we can refuse to
        // delete the only on-disk version of a plugin even when the pin is
        // stale or missing. Without this guard, a ghost `.active-version`
        // pointing at a non-existent version would mark the lone real
        // version as removable and the next `cleen cleanup --plugins`
        // would wipe it.
        let mut version_dirs: Vec<(String, PathBuf)> = Vec::new();
        for version_entry in fs::read_dir(&plugin_path)? {
            let version_entry = version_entry?;
            let version_path = version_entry.path();

            if !version_path.is_dir() {
                continue;
            }

            let version = version_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            version_dirs.push((version, version_path));
        }

        if version_dirs.len() <= 1 {
            // Single (or zero) on-disk version: nothing safe to remove here.
            continue;
        }

        for (version, version_path) in &version_dirs {
            // Skip active version
            if active_version.as_ref() == Some(version) {
                continue;
            }

            let size = calculate_dir_size(version_path).unwrap_or(0);

            print!("  Removing {}/{}... ", plugin_name, version);

            match fs::remove_dir_all(version_path) {
                Ok(()) => {
                    println!("done ({})", format_size(size));
                    removed_count += 1;
                    freed_bytes += size;
                }
                Err(e) => {
                    println!("failed: {}", e);
                }
            }
        }
    }

    if removed_count == 0 {
        println!("No inactive plugin versions to remove.");
    } else {
        println!();
        println!(
            "Cleanup complete: removed {} version(s), freed {}",
            removed_count,
            format_size(freed_bytes)
        );
    }

    Ok(())
}
