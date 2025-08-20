use thiserror::Error;
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, CleanManagerError>;

#[derive(Error, Debug)]
pub enum CleanManagerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Version '{version}' not found")]
    VersionNotFound { version: String },

    #[error("Version '{version}' is already installed")]
    VersionAlreadyInstalled { version: String },

    #[error("No version is currently active")]
    NoActiveVersion,

    #[error("Invalid version format: '{version}'")]
    InvalidVersion { version: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("GitHub API error: {message}")]
    GitHubError { message: String },

    #[error("Download failed: {url}")]
    DownloadError { url: String },

    #[error("Extraction failed: {path}")]
    ExtractionError { path: PathBuf },

    #[error("Home directory not found")]
    HomeDirectoryNotFound,

    #[error("Shell configuration error: {message}")]
    ShellError { message: String },

    #[error("Permission denied: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("Binary not found: {name}")]
    BinaryNotFound { name: String },

    #[error("Environment setup error: {message}")]
    EnvironmentError { message: String },

    #[error("Binary validation error: {message}")]
    ValidationError { message: String },
}

impl From<anyhow::Error> for CleanManagerError {
    fn from(error: anyhow::Error) -> Self {
        CleanManagerError::ShellError {
            message: error.to_string(),
        }
    }
}

impl CleanManagerError {
    pub fn config_error<S: Into<String>>(message: S) -> Self {
        CleanManagerError::ConfigError {
            message: message.into(),
        }
    }

    pub fn github_error<S: Into<String>>(message: S) -> Self {
        CleanManagerError::GitHubError {
            message: message.into(),
        }
    }

    pub fn shell_error<S: Into<String>>(message: S) -> Self {
        CleanManagerError::ShellError {
            message: message.into(),
        }
    }

    pub fn environment_error<S: Into<String>>(message: S) -> Self {
        CleanManagerError::EnvironmentError {
            message: message.into(),
        }
    }
}