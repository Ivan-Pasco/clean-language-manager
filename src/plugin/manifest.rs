use crate::error::{CleenError, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Plugin manifest structure matching plugin.toml format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMetadata,
    #[serde(default)]
    pub compatibility: PluginCompatibility,
    #[serde(default)]
    pub exports: PluginExports,
    #[serde(default)]
    pub dependencies: PluginDependencies,
}

/// Core plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub repository: Option<String>,
}

/// Compiler version compatibility requirements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginCompatibility {
    pub min_compiler_version: Option<String>,
    pub max_compiler_version: Option<String>,
}

/// Exported function names for plugin entry points
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginExports {
    #[serde(default = "default_expand")]
    pub expand: String,
    #[serde(default = "default_validate")]
    pub validate: String,
}

impl Default for PluginExports {
    fn default() -> Self {
        Self {
            expand: default_expand(),
            validate: default_validate(),
        }
    }
}

fn default_expand() -> String {
    "expand_block".to_string()
}

fn default_validate() -> String {
    "validate_block".to_string()
}

/// Dependencies on other plugins (planned feature)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginDependencies {
    #[serde(flatten)]
    pub plugins: std::collections::HashMap<String, String>,
}

impl PluginManifest {
    /// Load a plugin manifest from a file path
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Err(CleenError::PluginManifestNotFound {
                path: path.to_path_buf(),
            });
        }

        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse a plugin manifest from TOML content
    pub fn parse(content: &str) -> Result<Self> {
        toml::from_str(content).map_err(|e| CleenError::PluginManifestError {
            message: e.to_string(),
        })
    }

    /// Serialize the manifest to TOML format
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).map_err(|e| CleenError::PluginManifestError {
            message: e.to_string(),
        })
    }

    /// Save the manifest to a file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = self.to_toml()?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Create a new manifest with default values for a plugin name
    pub fn new(name: &str) -> Self {
        Self {
            plugin: PluginMetadata {
                name: name.to_string(),
                version: "0.1.0".to_string(),
                description: Some(format!("{} plugin for Clean Language", name)),
                author: None,
                license: Some("MIT".to_string()),
                repository: None,
            },
            compatibility: PluginCompatibility {
                min_compiler_version: Some("0.15.0".to_string()),
                max_compiler_version: None,
            },
            exports: PluginExports::default(),
            dependencies: PluginDependencies::default(),
        }
    }

    /// Validate the manifest has all required fields
    pub fn validate(&self) -> Result<()> {
        if self.plugin.name.is_empty() {
            return Err(CleenError::PluginManifestError {
                message: "Plugin name is required".to_string(),
            });
        }

        if self.plugin.version.is_empty() {
            return Err(CleenError::PluginManifestError {
                message: "Plugin version is required".to_string(),
            });
        }

        // Validate name format (alphanumeric, dots, hyphens)
        if !self.plugin.name.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_') {
            return Err(CleenError::PluginManifestError {
                message: "Plugin name can only contain alphanumeric characters, dots, hyphens, and underscores".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let content = r#"
[plugin]
name = "frame.web"
version = "1.0.0"
description = "Web framework plugin"
author = "Test Author"
license = "MIT"

[compatibility]
min_compiler_version = "0.15.0"

[exports]
expand = "expand_block"
validate = "validate_block"
"#;

        let manifest = PluginManifest::parse(content).unwrap();
        assert_eq!(manifest.plugin.name, "frame.web");
        assert_eq!(manifest.plugin.version, "1.0.0");
        assert_eq!(manifest.compatibility.min_compiler_version, Some("0.15.0".to_string()));
    }

    #[test]
    fn test_new_manifest() {
        let manifest = PluginManifest::new("my-plugin");
        assert_eq!(manifest.plugin.name, "my-plugin");
        assert_eq!(manifest.plugin.version, "0.1.0");
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_serialize_manifest() {
        let manifest = PluginManifest::new("test-plugin");
        let toml = manifest.to_toml().unwrap();
        assert!(toml.contains("name = \"test-plugin\""));
        assert!(toml.contains("version = \"0.1.0\""));
    }
}
