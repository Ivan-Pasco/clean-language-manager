use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, CleenError>;

#[derive(Error, Debug)]
pub enum CleenError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Version '{version}' not found")]
    VersionNotFound { version: String },

    #[error("Version '{version}' is already installed")]
    VersionAlreadyInstalled { version: String },

    #[error("No version is currently active")]
    #[allow(dead_code)]
    NoActiveVersion,

    #[error("Invalid version format: '{version}'")]
    InvalidVersion { version: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("GitHub API error: {message}")]
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    EnvironmentError { message: String },

    #[error("Binary validation error: {message}")]
    ValidationError { message: String },

    #[error("Update error: {message}")]
    UpdateError { message: String },
}

impl From<anyhow::Error> for CleenError {
    fn from(error: anyhow::Error) -> Self {
        CleenError::ShellError {
            message: error.to_string(),
        }
    }
}

impl CleenError {
    #[allow(dead_code)]
    pub fn config_error<S: Into<String>>(message: S) -> Self {
        CleenError::ConfigError {
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn github_error<S: Into<String>>(message: S) -> Self {
        CleenError::GitHubError {
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn shell_error<S: Into<String>>(message: S) -> Self {
        CleenError::ShellError {
            message: message.into(),
        }
    }

    #[allow(dead_code)]
    pub fn environment_error<S: Into<String>>(message: S) -> Self {
        CleenError::EnvironmentError {
            message: message.into(),
        }
    }
}
