use crate::core::config::Config;
use crate::error::{CleenError, Result};
use crate::plugin::manifest::PluginManifest;
use crate::plugin::registry;
use crate::plugin::scaffold;
use crate::plugin::{
    activate_plugin_version_root, get_plugin_versions, is_plugin_installed, list_installed_plugins,
    parse_plugin_specifier, remove_plugin,
};
use std::env;
use std::path::Path;
use std::process::Command;

/// Install a plugin from the registry or local source
pub fn install_plugin(specifier: &str) -> Result<()> {
    let mut config = Config::load()?;

    let (name, version) = parse_plugin_specifier(specifier);

    // Check if already installed
    if let Some(v) = &version {
        if is_plugin_installed(&config, &name, v) {
            return Err(CleenError::PluginAlreadyInstalled { name });
        }
    }

    // Try to install from registry
    registry::install_from_registry(&mut config, &name, version.as_deref())
}

/// Install a plugin from a local directory
pub fn install_local_plugin(path: &Path) -> Result<()> {
    let mut config = Config::load()?;
    registry::install_from_local(&mut config, path)
}

/// List all installed plugins
pub fn list_plugins() -> Result<()> {
    let config = Config::load()?;
    let plugins = list_installed_plugins(&config)?;

    if plugins.is_empty() {
        println!("No plugins installed");
        println!();
        println!("To install a plugin:");
        println!("  cleen plugin install <name>");
        println!();
        println!("To create a new plugin:");
        println!("  cleen plugin create <name>");
        return Ok(());
    }

    println!("Installed plugins:");
    println!();

    // Group by plugin name
    let mut current_name = String::new();
    for plugin in &plugins {
        if plugin.name != current_name {
            if !current_name.is_empty() {
                println!();
            }
            println!("  {}", plugin.name);
            current_name = plugin.name.clone();
        }

        let active = config.get_active_plugin_version(&plugin.name);
        let marker = if active == Some(&plugin.version) {
            "* "
        } else {
            "  "
        };

        let description = plugin
            .manifest
            .plugin
            .description
            .as_ref()
            .map(|d| format!(" - {}", d))
            .unwrap_or_default();

        println!("    {}{}{}", marker, plugin.version, description);
    }

    Ok(())
}

/// Create a new plugin project
pub fn create_plugin(name: &str) -> Result<()> {
    // Validate the name
    if name.is_empty() {
        return Err(CleenError::PluginManifestError {
            message: "Plugin name cannot be empty".to_string(),
        });
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        return Err(CleenError::PluginManifestError {
            message: "Plugin name can only contain alphanumeric characters, dots, hyphens, and underscores".to_string(),
        });
    }

    scaffold::create_plugin_project(name, None)
}

/// Build a plugin in the current directory
pub fn build_plugin() -> Result<()> {
    let current_dir = env::current_dir()?;
    let manifest_path = current_dir.join("plugin.toml");

    // Load and validate manifest
    let manifest = PluginManifest::load(&manifest_path)?;
    manifest.validate()?;

    println!("Building plugin '{}'...", manifest.plugin.name);

    // Check for source file
    let source_path = current_dir.join("src").join("main.cln");
    if !source_path.exists() {
        return Err(CleenError::PluginBuildError {
            message: format!("Source file not found: {}", source_path.display()),
        });
    }

    // Check if compiler is available
    let config = Config::load()?;
    let compiler_version = config
        .active_version
        .clone()
        .ok_or(CleenError::NoCompilerForPlugin)?;

    println!("Compiling src/main.cln...");

    // Get the compiler path
    let compiler_path = config.get_version_binary(&compiler_version);
    if !compiler_path.exists() {
        return Err(CleenError::BinaryNotFound {
            name: "cln".to_string(),
        });
    }

    // Run the compiler
    let output_path = current_dir.join("plugin.wasm");
    let output = Command::new(&compiler_path)
        .arg("compile")
        .arg(&source_path)
        .arg("-o")
        .arg(&output_path)
        .output();

    match output {
        Ok(result) => {
            if result.status.success() {
                // Get file size
                let size = std::fs::metadata(&output_path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                let size_kb = size as f64 / 1024.0;

                println!("Generated plugin.wasm ({:.1} KB)", size_kb);
                println!("Build successful");
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&result.stderr);
                let stdout = String::from_utf8_lossy(&result.stdout);
                Err(CleenError::PluginBuildError {
                    message: format!("Compilation failed:\n{}\n{}", stdout.trim(), stderr.trim()),
                })
            }
        }
        Err(e) => Err(CleenError::PluginBuildError {
            message: format!("Failed to run compiler: {}", e),
        }),
    }
}

/// Publish a plugin to the registry
pub fn publish_plugin() -> Result<()> {
    let current_dir = env::current_dir()?;
    let manifest_path = current_dir.join("plugin.toml");

    // Load and validate manifest
    let manifest = PluginManifest::load(&manifest_path)?;
    manifest.validate()?;

    println!(
        "Publishing {}@{}...",
        manifest.plugin.name, manifest.plugin.version
    );

    // Check for WASM file
    let wasm_path = current_dir.join("plugin.wasm");
    if !wasm_path.exists() {
        return Err(CleenError::PluginBuildError {
            message: "plugin.wasm not found. Run 'cleen plugin build' first.".to_string(),
        });
    }

    println!("Validating manifest...");
    println!("Validating plugin.wasm...");

    // Try to publish
    let client = registry::RegistryClient::new();
    client.publish(&manifest, &wasm_path)
}

/// Remove a plugin
pub fn remove_plugin_command(name: &str) -> Result<()> {
    let mut config = Config::load()?;

    // Check if plugin exists
    let plugin_dir = config.get_plugin_dir(name);
    if !plugin_dir.exists() {
        return Err(CleenError::PluginNotFound {
            name: name.to_string(),
        });
    }

    println!("Removing {}...", name);

    remove_plugin(&mut config, name)?;

    println!("Removed {}", plugin_dir.display());
    println!("Plugin {} removed successfully", name);

    Ok(())
}

/// Use a specific version of a plugin
pub fn use_plugin_version(name: &str, version: &str) -> Result<()> {
    let mut config = Config::load()?;

    // Check if version is installed
    if !is_plugin_installed(&config, name, version) {
        // List available versions
        let versions = get_plugin_versions(&config, name)?;
        if versions.is_empty() {
            return Err(CleenError::PluginNotFound {
                name: name.to_string(),
            });
        } else {
            println!("Version '{}' not installed for plugin '{}'", version, name);
            println!();
            println!("Installed versions:");
            for v in &versions {
                println!("  {}", v);
            }
            return Err(CleenError::PluginVersionNotFound {
                name: name.to_string(),
                version: version.to_string(),
            });
        }
    }

    config.set_active_plugin(name, version)?;
    activate_plugin_version_root(&config, name, version)?;

    println!("Now using {} version {}", name, version);

    Ok(())
}
