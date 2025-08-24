use crate::core::{config::Config, version::normalize};
use crate::error::{CleenError, Result};
use crate::utils::fs;
use std::path::Path;

pub struct ShimManager {
    config: Config,
}

impl ShimManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn create_shim(&self, version: &str) -> Result<()> {
        let clean_version = normalize::to_clean_version(version);
        let shim_path = self.config.get_shim_path();

        // Remove existing shim if it exists
        self.remove_shim()?;

        // Ensure bin directory exists
        fs::ensure_dir_exists(&self.config.get_bin_dir())?;

        // Create smart shim that checks for project versions
        self.create_smart_shim(&shim_path, &clean_version)?;

        println!("✅ Activated Clean Language version {clean_version}");

        Ok(())
    }

    pub fn remove_shim(&self) -> Result<()> {
        let shim_path = self.config.get_shim_path();

        if shim_path.exists() {
            std::fs::remove_file(&shim_path)?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_current_shim_target(&self) -> Result<Option<String>> {
        // Use the effective version (project-specific or global)
        Ok(self.config.get_effective_version())
    }

    /// Get the version that should be used for the current directory
    #[allow(dead_code)]
    pub fn get_effective_version(&self) -> Option<String> {
        self.config.get_effective_version()
    }

    /// Create a smart shim that checks for project-specific versions
    fn create_smart_shim(&self, shim_path: &Path, version: &str) -> Result<()> {
        // Find the actual binary path (checking both clean and v-prefixed versions)
        let binary_path = {
            let clean_path = self.config.get_version_binary(version);
            if clean_path.exists() {
                clean_path
            } else {
                let v_version = normalize::to_github_version(version);
                let v_path = self.config.get_version_binary(&v_version);
                if v_path.exists() {
                    v_path
                } else {
                    return Err(CleenError::VersionNotFound {
                        version: version.to_string(),
                    });
                }
            }
        };
        
        self.create_link(&binary_path, shim_path)?;
        Ok(())
    }

    #[cfg(unix)]
    fn create_link(&self, target: &Path, link: &Path) -> Result<()> {
        std::os::unix::fs::symlink(target, link)?;
        Ok(())
    }

    #[cfg(windows)]
    fn create_link(&self, target: &Path, link: &Path) -> Result<()> {
        // On Windows, copy the file instead of symlinking
        std::fs::copy(target, link)?;
        Ok(())
    }

    #[cfg(unix)]
    #[allow(dead_code)]
    fn resolve_link(&self, link: &Path) -> Result<std::path::PathBuf> {
        Ok(std::fs::read_link(link)?)
    }

    #[cfg(windows)]
    #[allow(dead_code)]
    fn resolve_link(&self, _link: &Path) -> Result<std::path::PathBuf> {
        // On Windows, we can't easily resolve what was copied
        // Return the link path itself
        Ok(_link.to_path_buf())
    }

    #[allow(dead_code)]
    pub fn verify_shim(&self) -> Result<bool> {
        let shim_path = self.config.get_shim_path();

        if !shim_path.exists() {
            return Ok(false);
        }

        // Check if the shim points to a valid version
        if let Ok(Some(version)) = self.get_current_shim_target() {
            let binary_path = self.config.get_version_binary(&version);
            return Ok(binary_path.exists() && fs::is_executable(&binary_path));
        }

        Ok(false)
    }

    #[allow(dead_code)]
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}
