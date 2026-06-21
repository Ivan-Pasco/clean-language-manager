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
        let lsp_shim_path = self.config.get_lsp_shim_path();
        let bin_dir = self.config.get_bin_dir();

        // Older installs created the shim as a bash wrapper script. On
        // macOS Sequoia those scripts inherit `com.apple.provenance` from
        // the cleen binary (itself downloaded via curl) and become
        // immutable — rm/chmod/rename-over all fail with EPERM even from
        // sudo. The only escape is to rename the parent dir. Detect and
        // perform that shuffle here so the subsequent symlink creation
        // path runs against a fresh, unlocked directory.
        let shim_name = shim_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("cln");
        let lsp_name = lsp_shim_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("clean-language-server");
        let _ = fs::evict_locked_shims(&bin_dir, &[shim_name, lsp_name]);

        // Remove existing shim if it exists
        self.remove_shim()?;

        // Ensure bin directory exists
        fs::ensure_dir_exists(&bin_dir)?;

        // Create smart shim that checks for project versions
        self.create_smart_shim(&shim_path, &clean_version)?;

        // Also link clean-language-server if present in this version
        self.create_lsp_shim(&clean_version)?;

        println!("✅ Activated Clean Language version {clean_version}");

        Ok(())
    }

    pub fn remove_shim(&self) -> Result<()> {
        // Use symlink-aware removal so dangling symlinks (left over from a
        // previously-removed version) don't leak past the cleanup.
        crate::utils::fs::remove_path_if_exists(&self.config.get_shim_path())?;
        crate::utils::fs::remove_path_if_exists(&self.config.get_lsp_shim_path())?;
        Ok(())
    }

    fn create_lsp_shim(&self, version: &str) -> Result<()> {
        let binary_path = {
            let clean_path = self.config.get_version_lsp_binary(version);
            if clean_path.exists() {
                clean_path
            } else {
                let v_version = normalize::to_github_version(version);
                let v_path = self.config.get_version_lsp_binary(&v_version);
                if v_path.exists() {
                    v_path
                } else {
                    return Ok(());
                }
            }
        };

        let shim_path = self.config.get_lsp_shim_path();
        self.create_wrapper_script(&binary_path, &shim_path)
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

        // Create a wrapper script instead of a direct link to prevent exec issues
        self.create_wrapper_script(&binary_path, shim_path)?;
        Ok(())
    }

    #[cfg(unix)]
    #[allow(dead_code)]
    fn create_link(&self, target: &Path, link: &Path) -> Result<()> {
        std::os::unix::fs::symlink(target, link)?;
        Ok(())
    }

    #[cfg(windows)]
    #[allow(dead_code)]
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

    #[cfg(unix)]
    fn create_wrapper_script(&self, binary_path: &Path, shim_path: &Path) -> Result<()> {
        // Symlink, not a bash wrapper. macOS Sequoia's `com.apple.provenance`
        // xattr makes wrapper scripts immutable to all user-level operations
        // (rm/chmod/xattr-c/rename-over all return EPERM, even via sudo).
        // Symlinks are not subject to that lock, so they can be atomically
        // rename-replaced indefinitely. The original wrapper script was
        // chosen "to prevent exec issues" (commit 35db590, Aug 2025) but no
        // concrete issue was documented, the prior code used symlinks
        // without trouble, and the frame shim has always used symlinks.
        fs::atomic_replace_symlink(shim_path, binary_path)?;
        Ok(())
    }

    #[cfg(windows)]
    fn create_wrapper_script(&self, binary_path: &Path, shim_path: &Path) -> Result<()> {
        // On Windows, create a .bat file
        let mut shim_path = shim_path.to_path_buf();
        shim_path.set_extension("bat");

        let script_content = format!("@echo off\n\"{}\" %*\n", binary_path.display());

        fs::atomic_write(&shim_path, script_content.as_bytes(), None)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_config(&self) -> &Config {
        &self.config
    }
}
