use crate::core::{config::Config, download::Downloader, github::GitHubClient};
use crate::error::{CleenError, Result};
use std::{env, fs, path::Path};

pub fn update_self_auto() -> Result<()> {
    println!("🔄 Checking for cleen updates...");

    let github = GitHubClient::new(None);
    let releases = github.get_releases("Ivan-Pasco", "clean-language-manager")?;

    if releases.is_empty() {
        println!("❌ No releases found for cleen");
        return Ok(());
    }

    let latest_release = &releases[0];
    let current_version = env!("CARGO_PKG_VERSION");

    if latest_release.tag_name.trim_start_matches('v') == current_version {
        println!("✅ cleen is up to date (version {current_version})");

        let mut config = Config::load()?;
        config.update_last_self_check_time()?;

        return Ok(());
    }

    println!(
        "🎉 New version available: {} (current: {})",
        latest_release.tag_name, current_version
    );
    println!();

    perform_auto_update(&latest_release)?;

    let mut config = Config::load()?;
    config.update_last_self_check_time()?;

    Ok(())
}

fn perform_auto_update(release: &crate::core::github::Release) -> Result<()> {
    println!("🚀 Starting automatic update to {}...", release.tag_name);

    // Get current binary path
    let current_exe = env::current_exe().map_err(|e| CleenError::UpdateError {
        message: format!("Failed to get current executable path: {}", e),
    })?;

    println!("📍 Current binary: {}", current_exe.display());

    // Find appropriate asset for current platform
    let platform_suffix = get_platform_suffix();
    println!("🔍 Looking for platform: {}", platform_suffix);

    let asset = find_update_asset(release, &platform_suffix)?;
    println!("📦 Found asset: {}", asset.name);

    // Create backup
    let backup_path = create_backup(&current_exe)?;
    println!("💾 Created backup: {}", backup_path.display());

    // Download new version
    let temp_dir = env::temp_dir().join(format!("cleen-update-{}", release.tag_name));
    fs::create_dir_all(&temp_dir)?;

    let downloader = Downloader::new();
    let download_path = temp_dir.join(&asset.name);

    println!("⬇️  Downloading {}...", asset.name);
    downloader
        .download_file(&asset.browser_download_url, &download_path)
        .map_err(|_| CleenError::UpdateError {
            message: "Failed to download update".to_string(),
        })?;

    // Extract or prepare binary
    let new_binary_path = prepare_new_binary(&download_path, &temp_dir, &asset.name)?;

    // Replace current binary
    replace_current_binary(&current_exe, &new_binary_path, &backup_path)?;

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);

    println!(
        "✅ Successfully updated cleen to version {}",
        release.tag_name
    );
    println!("🔄 Please restart your terminal session to use the new version");
    println!(
        "📝 The previous version has been backed up to: {}",
        backup_path.display()
    );

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

fn find_update_asset<'a>(
    release: &'a crate::core::github::Release,
    platform_suffix: &str,
) -> Result<&'a crate::core::github::Asset> {
    let binary_name = if cfg!(windows) { "cleen.exe" } else { "cleen" };

    // Look for platform-specific asset
    release
        .assets
        .iter()
        .find(|asset| {
            let name_lower = asset.name.to_lowercase();
            name_lower.contains(&platform_suffix.to_lowercase())
                && (name_lower.contains("cleen") || name_lower == binary_name)
        })
        .or_else(|| {
            // Fallback: look for any cleen binary
            release.assets.iter().find(|asset| {
                let name = &asset.name;
                name.contains("cleen") || name == binary_name
            })
        })
        .ok_or_else(|| {
            eprintln!("Available assets:");
            for asset in &release.assets {
                eprintln!("  • {}", asset.name);
            }
            CleenError::UpdateError {
                message: format!("No suitable binary found for platform {}", platform_suffix),
            }
        })
}

fn create_backup(current_exe: &Path) -> Result<std::path::PathBuf> {
    let backup_name = format!(
        "cleen-backup-{}.{}",
        chrono::Utc::now().format("%Y%m%d-%H%M%S"),
        if cfg!(windows) { "exe" } else { "bak" }
    );

    let backup_path = current_exe
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(backup_name);

    fs::copy(current_exe, &backup_path)?;
    Ok(backup_path)
}

fn prepare_new_binary(
    download_path: &Path,
    temp_dir: &Path,
    asset_name: &str,
) -> Result<std::path::PathBuf> {
    let binary_name = if cfg!(windows) { "cleen.exe" } else { "cleen" };

    if asset_name.ends_with(".tar.gz") || asset_name.ends_with(".zip") {
        println!("📦 Extracting archive...");
        let downloader = Downloader::new();
        downloader
            .extract_archive(download_path, temp_dir)
            .map_err(|_| CleenError::UpdateError {
                message: "Failed to extract archive".to_string(),
            })?;

        // Find the binary in the extracted files
        find_binary_in_extracted_dir(temp_dir, binary_name)
    } else {
        // Direct binary download
        let binary_path = temp_dir.join(binary_name);
        fs::copy(download_path, &binary_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        Ok(binary_path)
    }
}

fn find_binary_in_extracted_dir(dir: &Path, binary_name: &str) -> Result<std::path::PathBuf> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Ok(found) = find_binary_in_extracted_dir(&path, binary_name) {
                return Ok(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(binary_name) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&path)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&path, perms)?;
            }
            return Ok(path);
        }
    }

    Err(CleenError::UpdateError {
        message: format!("Binary '{}' not found in extracted archive", binary_name),
    })
}

fn replace_current_binary(
    current_exe: &Path,
    new_binary: &Path,
    _backup_path: &Path,
) -> Result<()> {
    println!("🔄 Replacing current binary...");

    #[cfg(windows)]
    {
        // On Windows, we can't replace a running executable directly
        // We need to use a different approach
        let temp_name = format!("{}.old", current_exe.to_string_lossy());
        let temp_path = Path::new(&temp_name);

        // Move current exe to temp location
        fs::rename(current_exe, temp_path)?;

        // Copy new binary to current location
        match fs::copy(new_binary, current_exe) {
            Ok(_) => {
                // Success - remove old binary
                let _ = fs::remove_file(temp_path);
            }
            Err(e) => {
                // Failed - restore original
                let _ = fs::rename(temp_path, current_exe);
                return Err(CleenError::UpdateError {
                    message: format!("Failed to replace binary: {}", e),
                });
            }
        }
    }

    #[cfg(unix)]
    {
        // On Unix, we can replace the binary directly
        fs::copy(new_binary, current_exe)?;

        // Ensure it's executable
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(current_exe)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(current_exe, perms)?;
    }

    Ok(())
}

pub fn check_for_updates() -> Result<()> {
    println!("🔄 Checking for Clean Language compiler updates...");

    let github = GitHubClient::new(None);
    let releases = github.get_releases("Ivan-Pasco", "clean-language-compiler")?;

    if releases.is_empty() {
        println!("❌ No releases found");
        return Ok(());
    }

    let config = Config::load()?;
    let latest_release = &releases[0];

    match &config.active_version {
        Some(current_version) => {
            if current_version == &latest_release.tag_name || current_version == "latest" {
                println!(
                    "✅ You're using the latest version: {}",
                    latest_release.tag_name
                );
            } else {
                println!(
                    "🎉 New version available: {} (current: {})",
                    latest_release.tag_name, current_version
                );
                println!();
                println!("To update:");
                println!("  cleen install latest");
                println!("  cleen use latest");
            }
        }
        None => {
            println!("⚠️  No version currently active");
            println!("Latest available: {}", latest_release.tag_name);
            println!();
            println!("To install:");
            println!("  cleen install latest");
            println!("  cleen use latest");
        }
    }

    let mut config = Config::load()?;
    config.update_last_check_time()?;

    Ok(())
}

pub fn check_updates_if_needed() -> Result<()> {
    let mut config = Config::load()?;

    if config.should_check_updates() && check_for_updates().is_ok() {
        let _ = config.update_last_check_time();
    }

    if config.should_check_self_updates() && update_self_auto().is_ok() {
        let _ = config.update_last_self_check_time();
    }

    Ok(())
}
