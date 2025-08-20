use crate::core::config::Config;
use crate::error::{CleanManagerError, Result};
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
        let binary_path = self.config.get_version_binary(version);
        let shim_path = self.config.get_shim_path();

        // Validate that the version binary exists
        if !binary_path.exists() {
            return Err(CleanManagerError::VersionNotFound {
                version: version.to_string(),
            });
        }

        // Remove existing shim if it exists
        self.remove_shim()?;

        // Ensure bin directory exists
        fs::ensure_dir_exists(&self.config.get_bin_dir())?;

        // Create smart shim that checks for project versions
        self.create_smart_shim(&shim_path)?;

        println!("✅ Activated Clean Language version {}", version);

        Ok(())
    }

    pub fn remove_shim(&self) -> Result<()> {
        let shim_path = self.config.get_shim_path();

        if shim_path.exists() {
            std::fs::remove_file(&shim_path)?;
        }

        Ok(())
    }

    pub fn get_current_shim_target(&self) -> Result<Option<String>> {
        // Use the effective version (project-specific or global)
        Ok(self.config.get_effective_version())
    }

    /// Get the version that should be used for the current directory
    pub fn get_effective_version(&self) -> Option<String> {
        self.config.get_effective_version()
    }

    /// Create a smart shim that checks for project-specific versions
    fn create_smart_shim(&self, shim_path: &Path) -> Result<()> {
        // For now, create a simple symlink to the global version
        // In a future enhancement, this could be a script that checks for .cleanversion
        if let Some(version) = &self.config.active_version {
            let binary_path = self.config.get_version_binary(version);
            self.create_link(&binary_path, shim_path)?;
        }

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
    fn resolve_link(&self, link: &Path) -> Result<std::path::PathBuf> {
        Ok(std::fs::read_link(link)?)
    }

    #[cfg(windows)]
    fn resolve_link(&self, _link: &Path) -> Result<std::path::PathBuf> {
        // On Windows, we can't easily resolve what was copied
        // Return the link path itself
        Ok(_link.to_path_buf())
    }

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

    pub fn get_config(&self) -> &Config {
        &self.config
    }
}
