use crate::core::{compatibility, config::Config, download::Downloader, github::GitHubClient};
use crate::error::{CleenError, Result};
use std::path::{Path, PathBuf};

const FRAME_REPO_OWNER: &str = "Ivan-Pasco";
const FRAME_REPO_NAME: &str = "cleen-framework";

/// Install Frame CLI
pub fn install_frame(version: Option<&str>, skip_compatibility_check: bool) -> Result<()> {
    let config = Config::load()?;

    // Determine version to install
    let frame_version = if let Some(v) = version {
        v.to_string()
    } else {
        // Auto-detect compatible version based on active compiler
        let compiler_version = config.active_version.as_ref().ok_or_else(|| {
            println!("⚠️  No compiler is currently active.");
            println!("   Frame CLI requires a Clean Language compiler.");
            println!();
            println!("To install a compiler first:");
            println!("   cleen install latest");
            println!("   cleen install 0.14.0");
            CleenError::NoCompilerForFrame
        })?;

        let matrix = compatibility::CompatibilityMatrix::new();
        match matrix.find_compatible_frame_version(compiler_version) {
            Some(v) => {
                println!("✓ Found compatible Frame CLI version: {v}");
                v
            }
            None => {
                println!("⚠️  No compatible Frame CLI version found for compiler {compiler_version}");
                println!("   Frame CLI 0.1.0 requires compiler >= 0.14.0");
                println!();
                println!("To upgrade your compiler:");
                println!("   cleen install 0.14.0");
                return Err(CleenError::FrameIncompatible {
                    frame_version: "0.1.0".to_string(),
                    required_compiler: "0.14.0".to_string(),
                    current_compiler: compiler_version.clone(),
                });
            }
        }
    };

    println!("Installing Frame CLI version: {frame_version}");

    // Check if version is already installed
    let version_dir = get_frame_version_dir(&config, &frame_version);
    if version_dir.exists() {
        return Err(CleenError::FrameVersionAlreadyInstalled {
            frame_version: frame_version.clone(),
        });
    }

    // Check compiler compatibility unless skipped
    if !skip_compatibility_check {
        if let Some(compiler_version) = &config.active_version {
            compatibility::check_frame_compatibility(compiler_version, &frame_version)?;
            println!("✓ Compatible with compiler {compiler_version}");
        } else {
            return Err(CleenError::NoCompilerForFrame);
        }
    }

    // Fetch releases from GitHub
    let github_client = GitHubClient::new(config.github_api_token.clone());
    println!("Fetching Frame CLI releases...");

    let releases = match github_client.get_releases(FRAME_REPO_OWNER, FRAME_REPO_NAME) {
        Ok(releases) => releases,
        Err(e) => {
            println!("⚠️  Unable to fetch releases from GitHub: {e}");
            println!("   Repository: https://github.com/{FRAME_REPO_OWNER}/{FRAME_REPO_NAME}/releases");
            return Ok(());
        }
    };

    if releases.is_empty() {
        println!("⚠️  No releases found in the repository.");
        println!("   Repository: https://github.com/{FRAME_REPO_OWNER}/{FRAME_REPO_NAME}/releases");
        return Ok(());
    }

    // Find the specified version (with or without 'v' prefix)
    let tag_name = format!("v{}", frame_version.trim_start_matches('v'));
    let release = releases.iter().find(|r| r.tag_name == tag_name).ok_or_else(|| {
        println!("Available Frame CLI versions:");
        for r in &releases {
            println!("  • {}", r.tag_name.trim_start_matches('v'));
        }
        CleenError::FrameVersionNotFound {
            frame_version: frame_version.clone(),
        }
    })?;

    // Find appropriate asset for current platform
    let platform_suffix = get_platform_suffix();
    println!("Looking for asset matching platform: {platform_suffix}");

    let asset = release
        .assets
        .iter()
        .find(|asset| {
            let name_lower = asset.name.to_lowercase();
            let matches_platform = name_lower.contains(&platform_suffix.to_lowercase())
                || name_lower.contains("universal")
                || name_lower.contains("any");
            let is_archive = name_lower.ends_with(".tar.gz") || name_lower.ends_with(".zip");
            matches_platform && is_archive
        })
        .or_else(|| {
            release.assets.iter().find(|asset| {
                let name_lower = asset.name.to_lowercase();
                let matches_platform = name_lower.contains(&platform_suffix.to_lowercase())
                    || name_lower.contains("universal")
                    || name_lower.contains("any");
                let is_binary = name_lower.contains("frame") && !name_lower.ends_with(".json");
                matches_platform && is_binary
            })
        })
        .ok_or_else(|| {
            println!("Available assets:");
            for asset in &release.assets {
                println!("  • {}", asset.name);
            }
            CleenError::BinaryNotFound {
                name: format!("Frame CLI asset for platform {platform_suffix}"),
            }
        })?;

    println!("Found asset: {}", asset.name);

    // Create temporary download directory
    let temp_dir = std::env::temp_dir().join(format!("cleen-frame-{frame_version}"));
    std::fs::create_dir_all(&temp_dir)?;

    // Download the asset
    let download_path = temp_dir.join(&asset.name);
    println!("Downloading {}...", asset.name);

    let downloader = Downloader::new();
    downloader
        .download_file(&asset.browser_download_url, &download_path)
        .map_err(|_e| CleenError::DownloadError {
            url: asset.browser_download_url.clone(),
        })?;

    // Extract to version directory
    std::fs::create_dir_all(&version_dir)?;

    if asset.name.ends_with(".tar.gz") || asset.name.ends_with(".zip") {
        println!("Extracting archive...");
        downloader
            .extract_archive(&download_path, &version_dir)
            .map_err(|_e| CleenError::ExtractionError {
                path: download_path.clone(),
            })?;
    } else {
        // Direct binary
        let binary_name = if cfg!(windows) { "frame.exe" } else { "frame" };
        let target_path = version_dir.join(binary_name);
        std::fs::copy(&download_path, &target_path)?;
    }

    // Find the extracted binary and ensure it's executable
    let binary_path = find_frame_binary_in_dir(&version_dir)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&binary_path, perms)?;
    }

    // Clean up temporary files
    std::fs::remove_dir_all(&temp_dir)?;

    // Validate the installed binary
    print!("🔍 Validating Frame CLI installation...");
    if let Err(e) = validate_frame_binary(&binary_path) {
        println!();
        eprintln!("⚠️  Warning: Installed Frame CLI may have issues: {e}");
        eprintln!("   The binary was installed but may not function correctly.");
    } else {
        println!(" ✅");
    }

    // Update config with Frame version
    let mut config = Config::load()?;
    config.frame_version = Some(frame_version.clone());
    config.save()?;

    // Update Frame symlink
    update_frame_symlink(&config, &frame_version)?;

    println!("✅ Successfully installed Frame CLI version {frame_version}");
    println!("   Binary location: {binary_path:?}");
    println!();
    println!("Frame CLI is now available:");
    println!("   frame --version");

    Ok(())
}

/// List installed Frame CLI versions
pub fn list_frame_versions(config: &Config) -> Result<Vec<String>> {
    let frame_dir = config.get_frame_versions_dir();

    if !frame_dir.exists() {
        return Ok(Vec::new());
    }

    let mut versions = Vec::new();

    for entry in std::fs::read_dir(&frame_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(version) = path.file_name().and_then(|n| n.to_str()) {
                versions.push(version.to_string());
            }
        }
    }

    versions.sort();
    Ok(versions)
}

/// Switch to a specific Frame CLI version
pub fn use_frame_version(version: &str) -> Result<()> {
    let mut config = Config::load()?;

    // Check if version is installed
    let version_dir = get_frame_version_dir(&config, version);
    if !version_dir.exists() {
        return Err(CleenError::FrameVersionNotFound {
            frame_version: version.to_string(),
        });
    }

    // Check compatibility with current compiler
    if let Some(compiler_version) = &config.active_version {
        if let Err(e) = compatibility::check_frame_compatibility(compiler_version, version) {
            eprintln!("⚠️  Warning: {e}");
            eprintln!("   Frame CLI may not work correctly with the current compiler.");
            eprintln!();
            eprintln!("Options:");
            eprintln!("  - Use a compatible compiler version");
            eprintln!("  - Continue anyway (not recommended)");
            eprintln!();
        }
    }

    // Update config
    config.frame_version = Some(version.to_string());
    config.save()?;

    // Update symlink
    update_frame_symlink(&config, version)?;

    println!("✅ Switched to Frame CLI version {version}");

    Ok(())
}

/// Uninstall a specific Frame CLI version
pub fn uninstall_frame_version(version: &str) -> Result<()> {
    let mut config = Config::load()?;

    // Check if version is installed
    let version_dir = get_frame_version_dir(&config, version);
    if !version_dir.exists() {
        return Err(CleenError::FrameVersionNotFound {
            frame_version: version.to_string(),
        });
    }

    // Check if this is the active version
    if config.frame_version.as_deref() == Some(version) {
        println!("⚠️  This is the currently active Frame CLI version.");
        println!("   Clearing active Frame version...");
        config.frame_version = None;
        config.save()?;

        // Remove symlink
        let shim_path = config.get_frame_shim_path();
        if shim_path.exists() {
            std::fs::remove_file(&shim_path)?;
        }
    }

    // Remove version directory
    std::fs::remove_dir_all(&version_dir)?;

    println!("✅ Uninstalled Frame CLI version {version}");

    Ok(())
}

/// Update Frame CLI symlink to point to the specified version
fn update_frame_symlink(config: &Config, version: &str) -> Result<()> {
    let binary_path = get_frame_binary_path(config, version);
    let shim_path = config.get_frame_shim_path();

    // Remove existing symlink if it exists
    if shim_path.exists() {
        std::fs::remove_file(&shim_path)?;
    }

    // Create new symlink
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&binary_path, &shim_path)?;
    }

    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(&binary_path, &shim_path)?;
    }

    Ok(())
}

/// Get the directory for a specific Frame CLI version
fn get_frame_version_dir(config: &Config, version: &str) -> PathBuf {
    config.get_frame_versions_dir().join(version)
}

/// Get the binary path for a specific Frame CLI version
fn get_frame_binary_path(config: &Config, version: &str) -> PathBuf {
    let binary_name = if cfg!(windows) { "frame.exe" } else { "frame" };
    get_frame_version_dir(config, version).join(binary_name)
}

/// Find Frame binary in a directory
fn find_frame_binary_in_dir(dir: &Path) -> Result<PathBuf> {
    let binary_name = if cfg!(windows) { "frame.exe" } else { "frame" };

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
            if let Ok(found) = find_frame_binary_in_dir(&path) {
                return Ok(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(binary_name) {
            return Ok(path);
        }
    }

    Err(CleenError::BinaryNotFound {
        name: binary_name.to_string(),
    })
}

/// Validate that the Frame CLI binary works
fn validate_frame_binary(binary_path: &Path) -> std::result::Result<(), String> {
    use std::process::Command;

    // Test 1: Check if binary exists
    if !binary_path.exists() {
        return Err("Binary file does not exist".to_string());
    }

    // Test 2: Try to run --version
    let version_output = Command::new(binary_path).args(["--version"]).output();

    match version_output {
        Ok(output) => {
            if !output.status.success() {
                return Err(format!(
                    "Binary failed to execute: exit code {}",
                    output.status.code().unwrap_or(-1)
                ));
            }

            let version_text = String::from_utf8_lossy(&output.stdout);
            if !version_text.to_lowercase().contains("frame") {
                return Err("Binary does not appear to be Frame CLI".to_string());
            }
        }
        Err(e) => {
            return Err(format!("Failed to execute binary: {e}"));
        }
    }

    Ok(())
}

/// Get platform suffix for downloads
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

    format!("{os}-{arch}")
}
