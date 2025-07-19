use crate::core::config::Config;
use crate::error::{Result, CleanManagerError};
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

        // Create symlink or copy based on platform capabilities
        self.create_link(&binary_path, &shim_path)?;

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
        let shim_path = self.config.get_shim_path();
        
        if !shim_path.exists() {
            return Ok(None);
        }

        // Try to resolve the target and find which version it points to
        let target = self.resolve_link(&shim_path)?;
        
        // Extract version from path
        if let Some(parent) = target.parent() {
            if let Some(version) = parent.file_name().and_then(|n| n.to_str()) {
                return Ok(Some(version.to_string()));
            }
        }
        
        Ok(None)
    }

    fn create_link(&self, target: &Path, link: &Path) -> Result<()> {
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target, link)?;
        }

        #[cfg(windows)]
        {
            // On Windows, try to create a symlink first, fallback to copy
            match std::os::windows::fs::symlink_file(target, link) {
                Ok(_) => {}
                Err(_) => {
                    // Fallback to copying the file
                    std::fs::copy(target, link)?;
                }
            }
        }

        Ok(())
    }

    fn resolve_link(&self, path: &Path) -> Result<std::path::PathBuf> {
        #[cfg(unix)]
        {
            std::fs::read_link(path).map_err(CleanManagerError::from)
        }

        #[cfg(windows)]
        {
            // On Windows, if it's a symlink, read it; otherwise return the path itself
            match std::fs::read_link(path) {
                Ok(target) => Ok(target),
                Err(_) => Ok(path.to_path_buf()),
            }
        }
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