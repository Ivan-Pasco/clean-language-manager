use crate::core::config::Config;
use crate::error::{CleenError, Result};
use crate::plugin::activate_plugin_version_root;
use crate::plugin::manifest::PluginManifest;
use crate::utils::fs as fs_utils;
use std::fs;
use std::path::Path;

/// Plugin registry base URL (placeholder for future implementation)
const REGISTRY_URL: &str = "https://plugins.cleanlang.org";

/// Plugin information from the registry
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub download_url: String,
    pub checksum: Option<String>,
}

/// Registry client for plugin operations
pub struct RegistryClient {
    base_url: String,
}

impl Default for RegistryClient {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistryClient {
    pub fn new() -> Self {
        Self {
            base_url: REGISTRY_URL.to_string(),
        }
    }

    /// Fetch information about a plugin from the registry
    pub fn get_plugin_info(&self, name: &str, _version: Option<&str>) -> Result<PluginInfo> {
        // For now, return a placeholder error since the registry is not yet implemented
        // In the future, this will make HTTP requests to the registry API

        Err(CleenError::PluginRegistryError {
            message: format!(
                "Plugin registry not yet available. Cannot fetch '{}' from {}",
                name, self.base_url
            ),
        })
    }

    /// List all available plugins from the registry
    #[allow(dead_code)]
    pub fn list_available(&self) -> Result<Vec<PluginInfo>> {
        // Placeholder for future implementation
        Err(CleenError::PluginRegistryError {
            message: format!("Plugin registry not yet available at {}", self.base_url),
        })
    }

    /// Search for plugins by name or description
    #[allow(dead_code)]
    pub fn search(&self, query: &str) -> Result<Vec<PluginInfo>> {
        // Placeholder for future implementation
        Err(CleenError::PluginRegistryError {
            message: format!(
                "Plugin registry search not yet available. Query: '{}'",
                query
            ),
        })
    }

    /// Publish a plugin to the registry
    pub fn publish(&self, _manifest: &PluginManifest, _wasm_path: &Path) -> Result<()> {
        // Placeholder for future implementation
        Err(CleenError::PluginRegistryError {
            message: "Plugin publishing not yet available. Registry is planned for future release."
                .to_string(),
        })
    }
}

/// Install a plugin from the registry
pub fn install_from_registry(config: &mut Config, name: &str, version: Option<&str>) -> Result<()> {
    let client = RegistryClient::new();

    // Try to get plugin info from registry
    match client.get_plugin_info(name, version) {
        Ok(info) => {
            download_and_install_plugin(config, &info)?;
            Ok(())
        }
        Err(e) => {
            // Registry not available, provide helpful message
            println!("Note: Plugin registry is not yet available.");
            println!();
            println!("To install a plugin locally:");
            println!("  1. Build the plugin: cleen plugin build");
            println!(
                "  2. Copy files to ~/.cleen/plugins/{}/{}/",
                name,
                version.unwrap_or("1.0.0")
            );
            println!();
            Err(e)
        }
    }
}

/// Download and install a plugin from its info
fn download_and_install_plugin(config: &mut Config, info: &PluginInfo) -> Result<()> {
    println!("Downloading {}@{}...", info.name, info.version);

    // Create plugin directory
    let plugin_dir = config.get_plugin_version_dir(&info.name, &info.version);
    fs_utils::ensure_dir_exists(&plugin_dir)?;

    // Download plugin archive
    // This is a placeholder - actual implementation would download from info.download_url

    println!("Extracting to {}...", plugin_dir.display());

    // Verify files
    let manifest_path = plugin_dir.join("plugin.toml");
    let wasm_path = plugin_dir.join("plugin.wasm");

    if !manifest_path.exists() {
        return Err(CleenError::PluginManifestNotFound {
            path: manifest_path,
        });
    }

    if !wasm_path.exists() {
        return Err(CleenError::PluginManifestError {
            message: "plugin.wasm not found in downloaded package".to_string(),
        });
    }

    // Verify checksum if available
    if info.checksum.is_some() {
        println!("Verifying checksum...");
        // TODO: Implement checksum verification
    }

    // Set as active version and activate root-level files
    config.set_active_plugin(&info.name, &info.version)?;
    activate_plugin_version_root(config, &info.name, &info.version)?;

    println!(
        "Plugin {}@{} installed successfully",
        info.name, info.version
    );

    Ok(())
}

/// Install a plugin from a local directory
pub fn install_from_local(config: &mut Config, source_dir: &Path) -> Result<()> {
    // Load manifest from source
    let manifest_path = source_dir.join("plugin.toml");
    let manifest = PluginManifest::load(&manifest_path)?;

    // Validate manifest
    manifest.validate()?;

    let name = &manifest.plugin.name;
    let version = &manifest.plugin.version;

    println!("Installing {} version {}...", name, version);

    // Check WASM exists
    let wasm_source = source_dir.join("plugin.wasm");
    if !wasm_source.exists() {
        return Err(CleenError::PluginBuildError {
            message: "plugin.wasm not found. Run 'cleen plugin build' first.".to_string(),
        });
    }

    // Create target directory
    let target_dir = config.get_plugin_version_dir(name, version);
    fs_utils::ensure_dir_exists(&target_dir)?;

    // Copy files
    let target_manifest = target_dir.join("plugin.toml");
    let target_wasm = target_dir.join("plugin.wasm");

    fs::copy(&manifest_path, &target_manifest)?;
    fs::copy(&wasm_source, &target_wasm)?;

    // Set as active version and activate root-level files
    config.set_active_plugin(name, version)?;
    activate_plugin_version_root(config, name, version)?;

    println!("Plugin {}@{} installed successfully", name, version);
    println!("Location: {}", target_dir.display());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_client_new() {
        let client = RegistryClient::new();
        assert_eq!(client.base_url, REGISTRY_URL);
    }

    #[test]
    fn test_registry_not_available() {
        let client = RegistryClient::new();
        let result = client.get_plugin_info("test-plugin", None);
        assert!(result.is_err());
    }
}
