use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
    pub prerelease: bool,
    pub draft: bool,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

pub struct GitHubClient {
    repo_owner: String,
    repo_name: String,
}

impl GitHubClient {
    pub fn new() -> Self {
        Self {
            repo_owner: "Ivan-Pasco".to_string(),
            repo_name: "clean-language-compiler".to_string(),
        }
    }

    pub fn get_releases(&self) -> Result<Vec<Release>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases",
            self.repo_owner, self.repo_name
        );

        let output = Command::new("curl")
            .arg("-s")
            .arg("-H")
            .arg("User-Agent: cleanmanager/0.1.0")
            .arg(&url)
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch releases: curl exited with status {:?}",
                output.status.code()
            ));
        }

        let response_text = String::from_utf8(output.stdout)?;
        let releases: Vec<Release> = serde_json::from_str(&response_text)?;
        Ok(releases)
    }

    pub fn get_latest_release(&self) -> Result<Release> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            self.repo_owner, self.repo_name
        );

        let output = Command::new("curl")
            .arg("-s")
            .arg("-H")
            .arg("User-Agent: cleanmanager/0.1.0")
            .arg(&url)
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch latest release: curl exited with status {:?}",
                output.status.code()
            ));
        }

        let response_text = String::from_utf8(output.stdout)?;
        let release: Release = serde_json::from_str(&response_text)?;
        Ok(release)
    }

    pub fn download_asset(&self, asset: &Asset, dest_path: &std::path::Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = dest_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let output = Command::new("curl")
            .arg("-L")  // Follow redirects
            .arg("-s")  // Silent
            .arg("-H")
            .arg("User-Agent: cleanmanager/0.1.0")
            .arg("-o")
            .arg(dest_path)
            .arg(&asset.browser_download_url)
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to download asset: curl exited with status {:?}",
                output.status.code()
            ));
        }

        Ok(())
    }
}