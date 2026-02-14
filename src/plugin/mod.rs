pub mod manifest;
pub mod registry;
pub mod scaffold;

use crate::core::config::Config;
use crate::error::{CleenError, Result};
use crate::utils::fs as fs_utils;
use manifest::PluginManifest;
use std::fs;

/// Represents an installed plugin with its metadata
#[derive(Debug, Clone)]
pub struct InstalledPlugin {
    pub name: String,
    pub version: String,
    pub manifest: PluginManifest,
}

/// List all installed plugins
pub fn list_installed_plugins(config: &Config) -> Result<Vec<InstalledPlugin>> {
    let plugins_dir = config.get_plugins_dir();

    if !plugins_dir.exists() {
        return Ok(Vec::new());
    }

    let mut plugins = Vec::new();

    for entry in fs::read_dir(&plugins_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let plugin_name = match path.file_name() {
            Some(name) => name.to_string_lossy().to_string(),
            None => continue,
        };

        // List versions for this plugin
        for version_entry in fs::read_dir(&path)? {
            let version_entry = version_entry?;
            let version_path = version_entry.path();

            if !version_path.is_dir() {
                continue;
            }

            let version = match version_path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            // Try to load manifest
            let manifest_path = version_path.join("plugin.toml");
            if manifest_path.exists() {
                if let Ok(manifest) = PluginManifest::load(&manifest_path) {
                    plugins.push(InstalledPlugin {
                        name: plugin_name.clone(),
                        version,
                        manifest,
                    });
                }
            }
        }
    }

    Ok(plugins)
}

/// Get all installed versions for a specific plugin
pub fn get_plugin_versions(config: &Config, name: &str) -> Result<Vec<String>> {
    let plugin_dir = config.get_plugin_dir(name);

    if !plugin_dir.exists() {
        return Ok(Vec::new());
    }

    let mut versions = Vec::new();

    for entry in fs::read_dir(&plugin_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Only include directories that contain a plugin.toml (actual version dirs)
            let manifest = path.join("plugin.toml");
            if manifest.exists() {
                if let Some(version) = path.file_name() {
                    versions.push(version.to_string_lossy().to_string());
                }
            }
        }
    }

    // Sort versions in descending order (newest first)
    versions.sort_by(|a, b| b.cmp(a));

    Ok(versions)
}

/// Check if a plugin version is installed
pub fn is_plugin_installed(config: &Config, name: &str, version: &str) -> bool {
    let manifest_path = config.get_plugin_manifest_path(name, version);
    let wasm_path = config.get_plugin_wasm_path(name, version);

    manifest_path.exists() && wasm_path.exists()
}

/// Remove a plugin (all versions)
pub fn remove_plugin(config: &mut Config, name: &str) -> Result<()> {
    let plugin_dir = config.get_plugin_dir(name);

    if !plugin_dir.exists() {
        return Err(CleenError::PluginNotFound {
            name: name.to_string(),
        });
    }

    // Remove the plugin directory
    fs::remove_dir_all(&plugin_dir)?;

    // Remove from active plugins
    config.remove_active_plugin(name)?;

    Ok(())
}

/// Remove a specific version of a plugin
#[allow(dead_code)]
pub fn remove_plugin_version(config: &mut Config, name: &str, version: &str) -> Result<()> {
    let version_dir = config.get_plugin_version_dir(name, version);

    if !version_dir.exists() {
        return Err(CleenError::PluginVersionNotFound {
            name: name.to_string(),
            version: version.to_string(),
        });
    }

    // Remove the version directory
    fs::remove_dir_all(&version_dir)?;

    // If this was the active version, handle root-level cleanup
    if config.get_active_plugin_version(name) == Some(&version.to_string()) {
        config.remove_active_plugin(name)?;
        clean_plugin_root_files(config, name)?;

        // If another version exists, activate it automatically
        let remaining_versions = get_plugin_versions(config, name)?;
        if let Some(latest) = remaining_versions.first() {
            config.set_active_plugin(name, latest)?;
            activate_plugin_version_root(config, name, latest)?;
        }
    }

    // If no versions remain, remove the plugin directory entirely
    let plugin_dir = config.get_plugin_dir(name);
    if plugin_dir.exists() {
        let remaining_versions = get_plugin_versions(config, name)?;
        if remaining_versions.is_empty() {
            fs::remove_dir_all(&plugin_dir)?;
        }
    }

    Ok(())
}

/// Activate a plugin version by copying its files to the plugin root directory.
/// This ensures the compiler can find the plugin files at the root level
/// (e.g., ~/.cleen/plugins/<name>/plugin.toml) regardless of versioned subdirectories.
pub fn activate_plugin_version_root(config: &Config, name: &str, version: &str) -> Result<()> {
    let version_dir = config.get_plugin_version_dir(name, version);
    let plugin_dir = config.get_plugin_dir(name);

    if !version_dir.exists() {
        return Err(CleenError::PluginVersionNotFound {
            name: name.to_string(),
            version: version.to_string(),
        });
    }

    // Copy all files from the version directory to the plugin root
    for entry in fs::read_dir(&version_dir)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dst_path = plugin_dir.join(&file_name);

        if src_path.is_dir() {
            // Remove existing directory at root level before copying
            if dst_path.exists() {
                fs::remove_dir_all(&dst_path)?;
            }
            fs_utils::copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    // Write .active-version marker file
    let active_version_path = plugin_dir.join(".active-version");
    fs::write(&active_version_path, version)?;

    Ok(())
}

/// Remove root-level activated files from a plugin directory,
/// keeping only the versioned subdirectories.
fn clean_plugin_root_files(config: &Config, name: &str) -> Result<()> {
    let plugin_dir = config.get_plugin_dir(name);

    if !plugin_dir.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(&plugin_dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Skip versioned subdirectories (they contain plugin.toml inside)
        if path.is_dir() {
            let manifest = path.join("plugin.toml");
            if manifest.exists() {
                continue;
            }
        }

        // Remove root-level files and non-version directories
        if path.is_dir() {
            fs::remove_dir_all(&path)?;
        } else {
            // Keep hidden files other than .active-version for safety,
            // but remove .active-version, plugin.toml, plugin.wasm, etc.
            if file_name == ".active-version"
                || !file_name.starts_with('.')
            {
                fs::remove_file(&path)?;
            }
        }
    }

    Ok(())
}

/// Parse a plugin specifier (e.g., "frame.web" or "frame.web@1.0.0")
pub fn parse_plugin_specifier(specifier: &str) -> (String, Option<String>) {
    if let Some(at_pos) = specifier.rfind('@') {
        let name = specifier[..at_pos].to_string();
        let version = specifier[at_pos + 1..].to_string();
        (name, Some(version))
    } else {
        (specifier.to_string(), None)
    }
}

/// Check if the current compiler version is compatible with a plugin
#[allow(dead_code)]
pub fn check_plugin_compatibility(config: &Config, manifest: &PluginManifest) -> Result<()> {
    let current_version = match &config.active_version {
        Some(v) => v.clone(),
        None => return Err(CleenError::NoCompilerForPlugin),
    };

    // Parse versions and compare
    if let Some(min_version) = &manifest.compatibility.min_compiler_version {
        if !version_satisfies(&current_version, min_version) {
            return Err(CleenError::PluginIncompatible {
                name: manifest.plugin.name.clone(),
                required: min_version.clone(),
                current: current_version,
            });
        }
    }

    Ok(())
}

/// Simple version comparison (current >= required)
#[allow(dead_code)]
fn version_satisfies(current: &str, required: &str) -> bool {
    // Strip 'v' prefix if present
    let current = current.trim_start_matches('v');
    let required = required.trim_start_matches('v');

    // Parse version parts
    let current_parts: Vec<u32> = current.split('.').filter_map(|p| p.parse().ok()).collect();
    let required_parts: Vec<u32> = required.split('.').filter_map(|p| p.parse().ok()).collect();

    // Compare each part
    for i in 0..std::cmp::max(current_parts.len(), required_parts.len()) {
        let curr = current_parts.get(i).copied().unwrap_or(0);
        let req = required_parts.get(i).copied().unwrap_or(0);

        if curr > req {
            return true;
        } else if curr < req {
            return false;
        }
    }

    true // Equal versions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_plugin_specifier() {
        let (name, version) = parse_plugin_specifier("frame.web");
        assert_eq!(name, "frame.web");
        assert_eq!(version, None);

        let (name, version) = parse_plugin_specifier("frame.web@1.0.0");
        assert_eq!(name, "frame.web");
        assert_eq!(version, Some("1.0.0".to_string()));

        let (name, version) = parse_plugin_specifier("my-plugin@2.1.0-beta");
        assert_eq!(name, "my-plugin");
        assert_eq!(version, Some("2.1.0-beta".to_string()));
    }

    #[test]
    fn test_version_satisfies() {
        assert!(version_satisfies("1.0.0", "1.0.0"));
        assert!(version_satisfies("1.1.0", "1.0.0"));
        assert!(version_satisfies("2.0.0", "1.0.0"));
        assert!(!version_satisfies("0.9.0", "1.0.0"));
        assert!(version_satisfies("v1.0.0", "1.0.0"));
        assert!(version_satisfies("1.0.0", "v1.0.0"));
    }
}
