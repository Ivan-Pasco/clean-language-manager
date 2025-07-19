use crate::core::{config::Config, github::GitHubClient, download::Downloader};
use crate::error::{Result, CleanManagerError};
use std::path::Path;

pub fn install_version(version: &str) -> Result<()> {
    println!("Installing Clean Language version: {}", version);
    
    let config = Config::load()?;
    let github_client = GitHubClient::new();
    let downloader = Downloader::new();
    
    // Check if version is already installed
    let version_dir = config.get_version_dir(version);
    if version_dir.exists() {
        return Err(CleanManagerError::VersionAlreadyInstalled {
            version: version.to_string(),
        });
    }
    
    // Resolve version (handle "latest")
    let actual_version = if version == "latest" {
        println!("Fetching latest release...");
        match github_client.get_latest_release() {
            Ok(release) => {
                println!("Latest version: {}", release.tag_name);
                release.tag_name
            },
            Err(e) => {
                println!("⚠️  Unable to fetch latest version from GitHub: {}", e);
                println!("   This may be because the repository doesn't have releases yet.");
                println!("   Please specify a specific version or check the repository:");
                println!("   https://github.com/Ivan-Pasco/clean-language-compiler/releases");
                return Ok(());
            }
        }
    } else {
        version.to_string()
    };
    
    println!("Resolved version: {}", actual_version);
    
    // Get releases and find the specified version
    println!("Fetching available releases...");
    let releases = match github_client.get_releases() {
        Ok(releases) => releases,
        Err(e) => {
            println!("⚠️  Unable to fetch releases from GitHub: {}", e);
            println!("   This may be because:");
            println!("   • The repository doesn't have releases yet");
            println!("   • Network connectivity issues");
            println!("   • GitHub API rate limiting");
            println!();
            println!("   Please check the repository manually:");
            println!("   https://github.com/Ivan-Pasco/clean-language-compiler/releases");
            return Ok(());
        }
    };
    
    if releases.is_empty() {
        println!("⚠️  No releases found in the repository.");
        println!("   The Clean Language compiler may still be in development.");
        println!("   Please check back later or follow the repository for updates:");
        println!("   https://github.com/Ivan-Pasco/clean-language-compiler/releases");
        return Ok(());
    }
    
    let release = releases
        .iter()
        .find(|r| r.tag_name == actual_version)
        .ok_or_else(|| {
            println!("Available versions:");
            for r in &releases {
                println!("  • {}", r.tag_name);
            }
            CleanManagerError::VersionNotFound {
                version: actual_version.clone(),
            }
        })?;
    
    // Find appropriate asset for current platform
    let platform_suffix = get_platform_suffix();
    println!("Looking for asset matching platform: {}", platform_suffix);
    
    let asset = release
        .assets
        .iter()
        .find(|asset| {
            let name_lower = asset.name.to_lowercase();
            name_lower.contains(&platform_suffix.to_lowercase()) ||
            name_lower.contains("universal") ||
            name_lower.contains("any")
        })
        .or_else(|| {
            // Fallback: try to find any executable asset
            release.assets.iter().find(|asset| {
                let name = &asset.name;
                name.ends_with(".exe") || 
                name.ends_with(".tar.gz") || 
                name.ends_with(".zip") ||
                name.contains("cln")
            })
        })
        .ok_or_else(|| {
            println!("Available assets:");
            for asset in &release.assets {
                println!("  • {}", asset.name);
            }
            CleanManagerError::BinaryNotFound {
                name: format!("Asset for platform {} (or universal binary)", platform_suffix),
            }
        })?;
    
    println!("Found asset: {}", asset.name);
    
    // Create temporary download directory
    let temp_dir = std::env::temp_dir().join(format!("cleanmanager-{}", actual_version));
    std::fs::create_dir_all(&temp_dir)?;
    
    // Download asset
    let download_path = temp_dir.join(&asset.name);
    github_client.download_asset(asset, &download_path).map_err(|e| CleanManagerError::DownloadError { url: asset.browser_download_url.clone() })?;
    
    // Extract to version directory
    std::fs::create_dir_all(&version_dir)?;
    
    if asset.name.ends_with(".tar.gz") || asset.name.ends_with(".zip") {
        println!("Extracting archive...");
        downloader.extract_archive(&download_path, &version_dir).map_err(|e| CleanManagerError::ExtractionError { path: download_path.clone() })?;
    } else {
        // Assume it's a direct binary
        let binary_name = if cfg!(windows) { "cln.exe" } else { "cln" };
        let target_path = version_dir.join(binary_name);
        std::fs::copy(&download_path, &target_path)?;
    }
    
    // Find the extracted binary and ensure it's executable
    let binary_path = find_binary_in_dir(&version_dir)?;
    
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms)?;
    }
    
    // Clean up temporary files
    std::fs::remove_dir_all(&temp_dir)?;
    
    println!("✅ Successfully installed Clean Language version {}", actual_version);
    println!("   Binary location: {:?}", binary_path);
    println!();
    println!("To use this version, run:");
    println!("   cleanmanager use {}", actual_version);
    
    Ok(())
}

fn get_platform_suffix() -> String {
    let os = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };
    
    let arch = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else {
        "unknown"
    };
    
    format!("{}-{}", os, arch)
}

fn find_binary_in_dir(dir: &Path) -> Result<std::path::PathBuf> {
    let binary_name = if cfg!(windows) { "cln.exe" } else { "cln" };
    
    // Look for binary in the root directory first
    let direct_path = dir.join(binary_name);
    if direct_path.exists() {
        return Ok(direct_path);
    }
    
    // Search recursively for the binary
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            if let Ok(found) = find_binary_in_dir(&path) {
                return Ok(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(binary_name) {
            return Ok(path);
        }
    }
    
    Err(CleanManagerError::BinaryNotFound {
        name: binary_name.to_string(),
    }
    .into())
}