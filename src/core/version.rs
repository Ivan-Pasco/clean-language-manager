use crate::core::config::Config;
use crate::error::{Result, CleanManagerError};
use crate::utils::fs;
use std::fs::read_dir;

#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: String,
    pub is_active: bool,
    pub binary_path: std::path::PathBuf,
    pub is_valid: bool,
}

pub struct VersionManager {
    config: Config,
}

impl VersionManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn list_installed_versions(&self) -> Result<Vec<VersionInfo>> {
        let versions_dir = self.config.get_versions_dir();
        
        if !versions_dir.exists() {
            return Ok(vec![]);
        }

        let mut versions = Vec::new();
        
        for entry in read_dir(&versions_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                if let Some(version_name) = path.file_name().and_then(|n| n.to_str()) {
                    let binary_path = self.config.get_version_binary(version_name);
                    let is_valid = binary_path.exists() && fs::is_executable(&binary_path);
                    let is_active = self.config.active_version.as_ref()
                        .map(|v| v == version_name)
                        .unwrap_or(false);
                    
                    versions.push(VersionInfo {
                        version: version_name.to_string(),
                        is_active,
                        binary_path,
                        is_valid,
                    });
                }
            }
        }

        // Sort versions
        versions.sort_by(|a, b| {
            // Try to parse as semantic versions, fallback to string comparison
            version_compare(&a.version, &b.version)
        });

        Ok(versions)
    }

    pub fn is_version_installed(&self, version: &str) -> bool {
        let binary_path = self.config.get_version_binary(version);
        binary_path.exists()
    }

    pub fn get_active_version(&self) -> Option<&String> {
        self.config.active_version.as_ref()
    }

    pub fn install_version(&self, version: &str, binary_path: &std::path::Path) -> Result<()> {
        if self.is_version_installed(version) {
            return Err(CleanManagerError::VersionAlreadyInstalled {
                version: version.to_string(),
            });
        }

        let target_dir = self.config.get_version_dir(version);
        let target_binary = self.config.get_version_binary(version);

        // Create version directory
        fs::ensure_dir_exists(&target_dir)?;

        // Copy binary to version directory
        fs::copy_file(binary_path, &target_binary)?;

        // Make it executable
        fs::make_executable(&target_binary)?;

        Ok(())
    }

    pub fn uninstall_version(&self, version: &str) -> Result<()> {
        if !self.is_version_installed(version) {
            return Err(CleanManagerError::VersionNotFound {
                version: version.to_string(),
            });
        }

        let version_dir = self.config.get_version_dir(version);
        fs::remove_dir_recursive(&version_dir)?;

        Ok(())
    }

    pub fn validate_version(&self, version: &str) -> Result<()> {
        if version.is_empty() {
            return Err(CleanManagerError::InvalidVersion {
                version: version.to_string(),
            });
        }

        // Basic validation - could be enhanced with semver parsing
        if version.contains("..") || version.contains('/') || version.contains('\\') {
            return Err(CleanManagerError::InvalidVersion {
                version: version.to_string(),
            });
        }

        Ok(())
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }
}

fn version_compare(a: &str, b: &str) -> std::cmp::Ordering {
    // Simple version comparison - could be enhanced with proper semver parsing
    use std::cmp::Ordering;

    let a_parts: Vec<&str> = a.split('.').collect();
    let b_parts: Vec<&str> = b.split('.').collect();

    for (a_part, b_part) in a_parts.iter().zip(b_parts.iter()) {
        match (a_part.parse::<u32>(), b_part.parse::<u32>()) {
            (Ok(a_num), Ok(b_num)) => {
                match a_num.cmp(&b_num) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            _ => return a_part.cmp(b_part),
        }
    }

    a_parts.len().cmp(&b_parts.len())
}