use crate::error::{CleenError, Result};
use crate::utils::fs;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub active_version: Option<String>,
    #[serde(default)]
    pub frame_version: Option<String>,
    pub cleen_dir: PathBuf,
    pub auto_cleanup: bool,
    pub github_api_token: Option<String>,
    #[serde(default = "default_true")]
    pub check_updates: bool,
    #[serde(default = "default_true")]
    pub auto_offer_frame: bool,
    pub last_update_check: Option<String>,
    pub last_self_update_check: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        let cleen_dir = get_cleen_dir().unwrap_or_else(|_| PathBuf::from(".cleen"));

        Config {
            active_version: None,
            frame_version: None,
            cleen_dir,
            auto_cleanup: false,
            github_api_token: None,
            check_updates: true,
            auto_offer_frame: true,
            last_update_check: None,
            last_self_update_check: None,
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let cleen_dir = get_cleen_dir()?;

        Ok(Config {
            active_version: None,
            frame_version: None,
            cleen_dir,
            auto_cleanup: false,
            github_api_token: std::env::var("GITHUB_TOKEN").ok(),
            check_updates: true,
            auto_offer_frame: true,
            last_update_check: None,
            last_self_update_check: None,
        })
    }

    pub fn load() -> Result<Self> {
        let config_path = get_config_path()?;

        if !config_path.exists() {
            let config = Self::new()?;
            config.save()?;
            return Ok(config);
        }

        let content = std::fs::read_to_string(&config_path)?;
        let config: Config = serde_json::from_str(&content)?;

        // Ensure directories exist
        fs::ensure_dir_exists(&config.cleen_dir)?;
        fs::ensure_dir_exists(&config.get_versions_dir())?;
        fs::ensure_dir_exists(&config.get_bin_dir())?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = get_config_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::ensure_dir_exists(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    pub fn set_active_version(&mut self, version: String) -> Result<()> {
        self.active_version = Some(version);
        self.save()
    }

    pub fn clear_active_version(&mut self) -> Result<()> {
        self.active_version = None;
        self.save()
    }

    /// Get the effective version to use, considering project-specific overrides
    pub fn get_effective_version(&self) -> Option<String> {
        // First, check for project-specific version file
        if let Some(project_version) = self.get_project_version() {
            return Some(project_version);
        }

        // Fall back to global active version
        self.active_version.clone()
    }

    /// Find project-specific version by looking for .cleanversion file
    pub fn get_project_version(&self) -> Option<String> {
        self.find_version_file_in_tree(&env::current_dir().ok()?)
    }

    /// Recursively search for .cleanlanguage/.cleanversion file in current directory and parents
    fn find_version_file_in_tree(&self, start_dir: &std::path::Path) -> Option<String> {
        let mut current_dir = start_dir.to_path_buf();

        loop {
            // Check for .cleanlanguage/.cleanversion file in current directory
            let clean_dir = current_dir.join(".cleanlanguage");
            let version_file = clean_dir.join(".cleanversion");

            if version_file.exists() {
                if let Ok(content) = std::fs::read_to_string(&version_file) {
                    let version = content.trim().to_string();
                    if !version.is_empty() {
                        return Some(version);
                    }
                }
            }

            // Move to parent directory
            match current_dir.parent() {
                Some(parent) => current_dir = parent.to_path_buf(),
                None => break,
            }
        }

        None
    }

    /// Create a .cleanlanguage/.cleanversion file in the current directory
    pub fn set_project_version(&self, version: &str) -> Result<()> {
        let current_dir = env::current_dir()?;
        let clean_dir = current_dir.join(".cleanlanguage");
        let version_file = clean_dir.join(".cleanversion");

        // Create .cleanlanguage directory if it doesn't exist
        std::fs::create_dir_all(&clean_dir)?;

        std::fs::write(&version_file, format!("{version}\n"))?;

        println!("✅ Created .cleanlanguage/.cleanversion file with version {version}");
        println!("   Project will now use Clean Language version {version}");

        Ok(())
    }

    pub fn get_versions_dir(&self) -> PathBuf {
        self.cleen_dir.join("versions")
    }

    pub fn get_bin_dir(&self) -> PathBuf {
        self.cleen_dir.join("bin")
    }

    pub fn get_version_dir(&self, version: &str) -> PathBuf {
        self.get_versions_dir().join(version)
    }

    pub fn get_version_binary(&self, version: &str) -> PathBuf {
        let binary_name = if cfg!(windows) { "cln.exe" } else { "cln" };
        self.get_version_dir(version).join(binary_name)
    }

    pub fn get_version_compile_options(&self, version: &str) -> PathBuf {
        self.get_version_dir(version).join("compile-options.json")
    }

    pub fn get_shim_path(&self) -> PathBuf {
        let binary_name = if cfg!(windows) { "cln.exe" } else { "cln" };
        self.get_bin_dir().join(binary_name)
    }

    pub fn should_check_updates(&self) -> bool {
        if !self.check_updates {
            return false;
        }

        match &self.last_update_check {
            None => true,
            Some(last_check) => {
                if let Ok(last_time) = last_check.parse::<i64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;

                    (now - last_time) > 86400
                } else {
                    true
                }
            }
        }
    }

    pub fn update_last_check_time(&mut self) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        self.last_update_check = Some(now);
        self.save()
    }

    pub fn should_check_self_updates(&self) -> bool {
        if !self.check_updates {
            return false;
        }

        match &self.last_self_update_check {
            None => true,
            Some(last_check) => {
                if let Ok(last_time) = last_check.parse::<i64>() {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;

                    (now - last_time) > 604800
                } else {
                    true
                }
            }
        }
    }

    pub fn update_last_self_check_time(&mut self) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string();

        self.last_self_update_check = Some(now);
        self.save()
    }

    // Frame CLI specific methods

    /// Get the Frame versions directory (~/.cleen/versions/frame/)
    pub fn get_frame_versions_dir(&self) -> PathBuf {
        self.get_versions_dir().join("frame")
    }

    /// Get the directory for a specific Frame CLI version
    pub fn get_frame_version_dir(&self, version: &str) -> PathBuf {
        self.get_frame_versions_dir().join(version)
    }

    /// Get the binary path for a specific Frame CLI version
    pub fn get_frame_version_binary(&self, version: &str) -> PathBuf {
        let binary_name = if cfg!(windows) { "frame.exe" } else { "frame" };
        self.get_frame_version_dir(version).join(binary_name)
    }

    /// Get the Frame CLI shim path
    pub fn get_frame_shim_path(&self) -> PathBuf {
        let binary_name = if cfg!(windows) { "frame.exe" } else { "frame" };
        self.get_bin_dir().join(binary_name)
    }
}

fn get_cleen_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".cleen"))
        .ok_or(CleenError::HomeDirectoryNotFound)
}

fn get_config_path() -> Result<PathBuf> {
    Ok(get_cleen_dir()?.join("config.json"))
}
