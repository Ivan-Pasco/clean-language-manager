use crate::error::{Result, CleanManagerError};
use crate::utils::fs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub active_version: Option<String>,
    pub cleanmanager_dir: PathBuf,
    pub auto_cleanup: bool,
    pub github_api_token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let cleanmanager_dir = get_cleanmanager_dir()
            .unwrap_or_else(|_| PathBuf::from(".cleanmanager"));

        Config {
            active_version: None,
            cleanmanager_dir,
            auto_cleanup: false,
            github_api_token: None,
        }
    }
}

impl Config {
    pub fn new() -> Result<Self> {
        let cleanmanager_dir = get_cleanmanager_dir()?;
        
        Ok(Config {
            active_version: None,
            cleanmanager_dir,
            auto_cleanup: false,
            github_api_token: std::env::var("GITHUB_TOKEN").ok(),
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
        fs::ensure_dir_exists(&config.cleanmanager_dir)?;
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
    fn find_version_file_in_tree(&self, start_dir: &PathBuf) -> Option<String> {
        let mut current_dir = start_dir.clone();
        
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
        
        std::fs::write(&version_file, format!("{}\n", version))?;
        
        println!("✅ Created .cleanlanguage/.cleanversion file with version {}", version);
        println!("   Project will now use Clean Language version {}", version);
        
        Ok(())
    }

    pub fn get_versions_dir(&self) -> PathBuf {
        self.cleanmanager_dir.join("versions")
    }

    pub fn get_bin_dir(&self) -> PathBuf {
        self.cleanmanager_dir.join("bin")
    }

    pub fn get_version_dir(&self, version: &str) -> PathBuf {
        self.get_versions_dir().join(version)
    }

    pub fn get_version_binary(&self, version: &str) -> PathBuf {
        let binary_name = if cfg!(windows) { "cln.exe" } else { "cln" };
        self.get_version_dir(version).join(binary_name)
    }

    pub fn get_shim_path(&self) -> PathBuf {
        let binary_name = if cfg!(windows) { "cln.exe" } else { "cln" };
        self.get_bin_dir().join(binary_name)
    }
}

fn get_cleanmanager_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".cleanmanager"))
        .ok_or(CleanManagerError::HomeDirectoryNotFound)
}

fn get_config_path() -> Result<PathBuf> {
    Ok(get_cleanmanager_dir()?.join("config.json"))
}