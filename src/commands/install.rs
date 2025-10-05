use crate::core::{config::Config, download::Downloader, github::GitHubClient, version::normalize};
use crate::error::{CleenError, Result};
use std::path::Path;

pub fn install_version(version: &str) -> Result<()> {
    println!("Installing Clean Language version: {version}");

    let config = Config::load()?;
    let github_client = GitHubClient::new(config.github_api_token.clone());
    let downloader = Downloader::new();

    // Resolve version (handle "latest") first and normalize to GitHub format
    let github_version = if version == "latest" {
        println!("Fetching latest release...");
        match github_client.get_latest_release("Ivan-Pasco", "clean-language-compiler") {
            Ok(release) => {
                println!("Latest version: {}", release.tag_name);
                release.tag_name
            }
            Err(e) => {
                println!("⚠️  Unable to fetch latest version from GitHub: {e}");
                println!("   This may be because the repository doesn't have releases yet.");
                println!("   Please specify a specific version or check the repository:");
                println!("   https://github.com/Ivan-Pasco/clean-language-compiler/releases");
                return Ok(());
            }
        }
    } else {
        normalize::to_github_version(version)
    };

    // Normalize to clean version for local storage
    let clean_version = normalize::to_clean_version(&github_version);

    println!("Resolved version: {clean_version}");

    // Check if version is already installed (using clean version for storage)
    let version_dir = config.get_version_dir(&clean_version);
    if version_dir.exists() {
        return Err(CleenError::VersionAlreadyInstalled {
            version: clean_version.clone(),
        });
    }

    // Get releases and find the specified version
    println!("Fetching available releases...");
    let releases = match github_client.get_releases("Ivan-Pasco", "clean-language-compiler") {
        Ok(releases) => releases,
        Err(e) => {
            println!("⚠️  Unable to fetch releases from GitHub: {e}");
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
        .find(|r| r.tag_name == github_version)
        .ok_or_else(|| {
            println!("Available versions:");
            for r in &releases {
                println!("  • {}", normalize::to_clean_version(&r.tag_name));
            }
            CleenError::VersionNotFound {
                version: clean_version.clone(),
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
            name_lower.contains(&platform_suffix.to_lowercase())
                || name_lower.contains("universal")
                || name_lower.contains("any")
        })
        .or_else(|| {
            // Fallback: try to find any executable asset
            release.assets.iter().find(|asset| {
                let name = &asset.name;
                name.ends_with(".exe")
                    || name.ends_with(".tar.gz")
                    || name.ends_with(".zip")
                    || name.contains("cln")
            })
        })
        .ok_or_else(|| {
            println!("Available assets:");
            for asset in &release.assets {
                println!("  • {}", asset.name);
            }
            CleenError::BinaryNotFound {
                name: format!("Asset for platform {platform_suffix} (or universal binary)"),
            }
        })?;

    println!("Found asset: {}", asset.name);

    // Create temporary download directory
    let temp_dir = std::env::temp_dir().join(format!("cleen-{clean_version}"));
    std::fs::create_dir_all(&temp_dir)?;

    // Download the asset
    let download_path = temp_dir.join(&asset.name);
    println!("Downloading {}...", asset.name);
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

    // compile-options.json is stored per-version in the version directory
    // The extraction already placed it there, just verify and inform the user
    let options_path = version_dir.join("compile-options.json");
    if options_path.exists() {
        println!("✓ Found compile-options.json for version {clean_version}");
    } else {
        // This is just informational, not an error, since older releases may not have this file
        println!("ℹ️  Note: compile-options.json not found in release package");
        println!("   This is expected for compiler versions before dynamic options support.");
    }

    // Clean up temporary files
    std::fs::remove_dir_all(&temp_dir)?;

    // Validate the installed binary works correctly
    print!("🔍 Validating installation...");
    if let Err(e) = validate_installed_binary(&binary_path) {
        println!();
        eprintln!("⚠️  Warning: Installed binary may have issues: {e}");
        eprintln!("   The binary was installed but may not function correctly.");
        eprintln!("   You may need to use a different version or compile from source.");
    } else {
        println!(" ✅");
    }

    println!("✅ Successfully installed Clean Language version {clean_version}");
    println!("   Binary location: {binary_path:?}");
    println!();
    println!("To use this version, run:");
    println!("   cleen use {clean_version}");

    Ok(())
}

fn validate_installed_binary(binary_path: &std::path::Path) -> std::result::Result<(), String> {
    use std::process::Command;

    // Test 1: Check if binary exists and is executable
    if !binary_path.exists() {
        return Err("Binary file does not exist".to_string());
    }

    // Test 2: Check if binary can show version (basic execution test)
    let version_output = Command::new(binary_path).args(["version"]).output();

    match version_output {
        Ok(output) => {
            if !output.status.success() {
                return Err(format!(
                    "Binary failed to execute: exit code {}",
                    output.status.code().unwrap_or(-1)
                ));
            }

            let version_text = String::from_utf8_lossy(&output.stdout);
            if !version_text.contains("Clean Language Compiler") {
                return Err("Binary does not appear to be Clean Language compiler".to_string());
            }
        }
        Err(e) => {
            return Err(format!("Failed to execute binary: {e}"));
        }
    }

    // Test 3: Try to compile a simple test program
    let test_program = r#"start()
	print("test")"#;

    // Create a temporary test file
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("cleen_test.cln");
    let test_wasm = temp_dir.join("cleen_test.wasm");

    // Write test program
    if let Err(e) = std::fs::write(&test_file, test_program) {
        return Err(format!("Failed to create test file: {e}"));
    }

    // Try to compile
    let compile_result = Command::new(binary_path)
        .args([
            "compile",
            test_file.to_str().unwrap(),
            test_wasm.to_str().unwrap(),
        ])
        .output();

    // Clean up test files
    let _ = std::fs::remove_file(&test_file);
    let _ = std::fs::remove_file(&test_wasm);

    match compile_result {
        Ok(output) => {
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Compilation test failed: {stderr}"));
            }
        }
        Err(e) => {
            return Err(format!("Failed to run compilation test: {e}"));
        }
    }

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

    format!("{os}-{arch}")
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

    Err(CleenError::BinaryNotFound {
        name: binary_name.to_string(),
    })
}
