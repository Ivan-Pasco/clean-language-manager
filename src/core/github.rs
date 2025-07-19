use anyhow::Result;

#[derive(Debug)]
pub struct Release {
    pub tag_name: String,
    pub name: String,
    pub assets: Vec<Asset>,
}

#[derive(Debug)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
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
        // TODO: Implement GitHub API calls
        Ok(vec![])
    }

    pub fn get_latest_release(&self) -> Result<Release> {
        // TODO: Implement getting latest release
        Err(anyhow::anyhow!("Not implemented"))
    }
}