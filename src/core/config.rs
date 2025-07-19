use crate::error::{Result, CleanManagerError};
use crate::utils::fs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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