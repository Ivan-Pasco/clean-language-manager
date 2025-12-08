use crate::core::{compatibility, config::Config, download::Downloader, github::GitHubClient};
use crate::error::{CleenError, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// PID file location for the server
fn get_pid_file_path() -> PathBuf {
    std::env::temp_dir().join("cleen-frame-server.pid")
}

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
                println!(
                    "⚠️  No compatible Frame CLI version found for compiler {compiler_version}"
                );
                println!("   Frame CLI requires compiler >= 0.14.0");
                println!("   • Frame 1.0.0 requires compiler >= 0.14.0");
                println!("   • Frame 2.0.0 requires compiler >= 0.16.0");
                println!();
                println!("To upgrade your compiler:");
                println!("   cleen install latest");
                return Err(CleenError::FrameIncompatible {
                    frame_version: "1.0.0+".to_string(),
                    required_compiler: "0.14.0".to_string(),
                    current_compiler: compiler_version.clone(),
                });
            }
        }
    };

    // Check if version is already installed
    let version_dir = get_frame_version_dir(&config, &frame_version);
    if version_dir.exists() {
        println!("✓ Frame CLI version {frame_version} is already installed");

        // Ensure this version is set as active
        let mut config = Config::load()?;
        if config.frame_version.as_deref() != Some(&frame_version) {
            config.frame_version = Some(frame_version.clone());
            config.save()?;
            update_frame_symlink(&config, &frame_version)?;
            println!("  Activated Frame CLI version {frame_version}");
        }

        return Ok(());
    }

    println!("Installing Frame CLI version: {frame_version}");

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
            println!(
                "   Repository: https://github.com/{FRAME_REPO_OWNER}/{FRAME_REPO_NAME}/releases"
            );
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
    let release = releases
        .iter()
        .find(|r| r.tag_name == tag_name)
        .ok_or_else(|| {
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

    // Auto-install Clean Server if not already installed
    let config = Config::load()?;
    if config.server_version.is_none() {
        println!("📦 Installing Clean Server (required for running Frame applications)...");
        println!();
        if let Err(e) = crate::core::server::install_server(None) {
            println!("⚠️  Warning: Could not auto-install Clean Server: {e}");
            println!("   You can install it manually with: cleen server install");
        }
        println!();
    }

    println!("Frame CLI is now available:");
    println!("   frame --version");
    println!();
    println!("To create a new project:");
    println!("   cleen frame new myapp");

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

/// Start a Frame development server
///
/// This function:
/// 1. Compiles the .cln source file to WASM using the Clean Language compiler
/// 2. Starts the frame-runtime with the compiled WASM file
pub fn serve_application(input: &str, port: u16, host: &str, debug: bool) -> Result<()> {
    let config = Config::load()?;

    // Check if a server is already running
    let pid_file = get_pid_file_path();
    if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                // Check if process is still running
                #[cfg(unix)]
                {
                    let status = Command::new("kill").args(["-0", &pid.to_string()]).output();
                    if status.map(|o| o.status.success()).unwrap_or(false) {
                        println!("⚠️  A Frame server is already running (PID: {pid})");
                        println!("   Use 'cleen frame stop' to stop it first");
                        return Ok(());
                    }
                }
            }
        }
        // Clean up stale PID file
        let _ = std::fs::remove_file(&pid_file);
    }

    // Verify input file exists
    let input_path = Path::new(input);
    if !input_path.exists() {
        return Err(CleenError::FileNotFound {
            path: input.to_string(),
        });
    }

    // Find the Clean Language compiler
    let cln_path = config.get_shim_path();
    if !cln_path.exists() {
        println!("⚠️  Clean Language compiler not found");
        println!("   Install it with: cleen install latest");
        return Err(CleenError::NoActiveVersion);
    }

    // Find the frame-runtime
    // It should be installed alongside Frame CLI or in the framework's runtime
    let runtime_path = find_frame_runtime(&config)?;

    // Create output WASM path in temp directory
    let wasm_path = std::env::temp_dir().join("cleen-serve-app.wasm");

    // Compile the source file
    println!("📦 Compiling {}...", input);
    let compile_output = Command::new(&cln_path)
        .args(["compile", input, "-o"])
        .arg(&wasm_path)
        .arg("--plugins")
        .output()
        .map_err(|e| CleenError::CompilationFailed {
            message: format!("Failed to run compiler: {e}"),
        })?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        println!("❌ Compilation failed:");
        println!("{stderr}");
        return Err(CleenError::CompilationFailed {
            message: stderr.to_string(),
        });
    }

    println!("✅ Compilation successful");

    // Set environment variables for the server
    let mut cmd = Command::new(&runtime_path);
    cmd.arg(&wasm_path);
    cmd.env("FRAME_PORT", port.to_string());
    cmd.env("FRAME_HOST", host);

    if debug {
        cmd.env("RUST_LOG", "debug");
    }

    println!();
    println!("🚀 Starting Frame development server...");
    println!("   Listening on http://{}:{}", host, port);
    println!();
    println!("   Press Ctrl+C to stop the server");
    println!();

    // Run the server in foreground (blocks until Ctrl+C)
    let mut child = cmd.spawn().map_err(|e| CleenError::ServerStartFailed {
        message: format!("Failed to start frame-runtime: {e}"),
    })?;

    // Save PID
    let pid = child.id();
    std::fs::write(&pid_file, pid.to_string())?;

    // Wait for the process (this blocks)
    let status = child.wait().map_err(|e| CleenError::ServerStartFailed {
        message: format!("Server exited with error: {e}"),
    })?;

    // Clean up PID file
    let _ = std::fs::remove_file(&pid_file);

    if !status.success() {
        println!("⚠️  Server exited with status: {:?}", status.code());
    } else {
        println!("Server stopped");
    }

    Ok(())
}

/// Stop a running Frame development server
pub fn stop_server() -> Result<()> {
    let pid_file = get_pid_file_path();

    if !pid_file.exists() {
        println!("No Frame server is currently running");
        return Ok(());
    }

    let pid_str = std::fs::read_to_string(&pid_file).map_err(|e| CleenError::IoError {
        message: format!("Failed to read PID file: {e}"),
    })?;

    let pid: i32 = pid_str.trim().parse().map_err(|_| CleenError::IoError {
        message: "Invalid PID in file".to_string(),
    })?;

    println!("Stopping Frame server (PID: {pid})...");

    #[cfg(unix)]
    {
        let output = Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                println!("✅ Server stopped successfully");
            }
            Ok(_) => {
                // Process might already be dead
                println!("⚠️  Process may have already stopped");
            }
            Err(e) => {
                println!("⚠️  Failed to stop server: {e}");
            }
        }
    }

    #[cfg(windows)]
    {
        let output = Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output();

        match output {
            Ok(o) if o.status.success() => {
                println!("✅ Server stopped successfully");
            }
            Ok(_) => {
                println!("⚠️  Process may have already stopped");
            }
            Err(e) => {
                println!("⚠️  Failed to stop server: {e}");
            }
        }
    }

    // Clean up PID file
    let _ = std::fs::remove_file(&pid_file);

    Ok(())
}

/// Find the frame-runtime binary
fn find_frame_runtime(config: &Config) -> Result<PathBuf> {
    // First, check if it's in the active Frame CLI version directory
    if let Some(frame_version) = &config.frame_version {
        let version_dir = config.get_frame_versions_dir().join(frame_version);

        // Look for frame-runtime in the version directory
        let runtime_name = if cfg!(windows) {
            "frame-runtime.exe"
        } else {
            "frame-runtime"
        };

        let runtime_path = version_dir.join(runtime_name);
        if runtime_path.exists() {
            return Ok(runtime_path);
        }

        // Also check in subdirectories
        if let Ok(found) = find_binary_in_dir(&version_dir, runtime_name) {
            return Ok(found);
        }
    }

    // Check if frame-runtime is in PATH
    if let Ok(path) = which::which("frame-runtime") {
        return Ok(path);
    }

    // Check common installation locations
    let home = dirs::home_dir().ok_or(CleenError::BinaryNotFound {
        name: "home directory".to_string(),
    })?;

    let common_paths = [
        home.join(".cleen").join("bin").join("frame-runtime"),
        home.join(".local").join("bin").join("frame-runtime"),
        PathBuf::from("/usr/local/bin/frame-runtime"),
    ];

    for path in common_paths {
        if path.exists() {
            return Ok(path);
        }
    }

    Err(CleenError::BinaryNotFound {
        name: "frame-runtime".to_string(),
    })
}

/// Find a binary in a directory (recursive)
fn find_binary_in_dir(dir: &Path, name: &str) -> Result<PathBuf> {
    let direct_path = dir.join(name);
    if direct_path.exists() {
        return Ok(direct_path);
    }

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Ok(found) = find_binary_in_dir(&path, name) {
                return Ok(found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
            return Ok(path);
        }
    }

    Err(CleenError::BinaryNotFound {
        name: name.to_string(),
    })
}

/// Create a new Frame project
///
/// Templates:
/// - `api`: API-only backend server
/// - `web`: Full-stack web application (frontend + backend)
/// - `minimal`: Bare minimum single-file project
pub fn create_project(name: &str, template: &str, port: u16) -> Result<()> {
    let project_dir = Path::new(name);

    // Check if directory already exists
    if project_dir.exists() {
        return Err(CleenError::ProjectAlreadyExists {
            name: name.to_string(),
        });
    }

    println!("Creating Frame project '{}'...", name);

    match template {
        "api" => create_api_template(name, port)?,
        "web" => create_web_template(name, port)?,
        "minimal" => create_minimal_template(name, port)?,
        _ => {
            return Err(CleenError::InvalidTemplate {
                template: template.to_string(),
            });
        }
    }

    // Create .cleanlanguage/.cleanversion file
    let cleanversion_dir = project_dir.join(".cleanlanguage");
    std::fs::create_dir_all(&cleanversion_dir)?;

    let config = Config::load()?;
    if let Some(version) = &config.active_version {
        std::fs::write(cleanversion_dir.join(".cleanversion"), version)?;
    }

    println!();
    println!("✅ Project '{}' created successfully!", name);
    println!();
    println!("Next steps:");
    println!("   cd {}", name);
    println!("   cleen frame serve");
    println!();
    println!("Your server will be available at http://127.0.0.1:{}", port);

    Ok(())
}

/// Create API template (backend only)
fn create_api_template(name: &str, port: u16) -> Result<()> {
    let project_dir = Path::new(name);

    // Create directory structure
    std::fs::create_dir_all(project_dir.join("app/api"))?;
    std::fs::create_dir_all(project_dir.join("config"))?;
    std::fs::create_dir_all(project_dir.join("dist"))?;

    // Create app/api/main.cln
    let main_cln = format!(
        r#"// {name} - API Server
// Created with Frame Framework

import:
	frame.web

endpoints:
	GET /api/hello:
		return "Hello from {name}!"

	GET /api/health:
		return "OK"

	GET /api/users:
		return "[]"

	GET /api/users/:id:
		string id = req.params.id
		return "User ID: " + id

	POST /api/users:
		return "User created"
"#,
        name = name
    );
    std::fs::write(project_dir.join("app/api/main.cln"), main_cln)?;

    // Create config/app.cln
    let config_cln = format!(
        r#"// Application Configuration
// {name}

config:
    name = "{name}"
    version = "0.1.0"
    environment = "development"

server:
    port = {port}
    host = "127.0.0.1"
"#,
        name = name,
        port = port
    );
    std::fs::write(project_dir.join("config/app.cln"), config_cln)?;

    // Create frame.toml
    let frame_toml = format!(
        r#"[project]
name = "{name}"
version = "0.1.0"
description = "A Frame API server"

[server]
port = {port}
host = "127.0.0.1"
entry = "app/api/main.cln"

[plugins]
frame.web = "1.0.0"
"#,
        name = name,
        port = port
    );
    std::fs::write(project_dir.join("frame.toml"), frame_toml)?;

    // Create .gitignore
    let gitignore = r#"# Build output
dist/
*.wasm

# Dependencies
node_modules/

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
"#;
    std::fs::write(project_dir.join(".gitignore"), gitignore)?;

    Ok(())
}

/// Create web template (full-stack)
fn create_web_template(name: &str, port: u16) -> Result<()> {
    let project_dir = Path::new(name);

    // Create full directory structure from spec
    std::fs::create_dir_all(project_dir.join("app/api"))?;
    std::fs::create_dir_all(project_dir.join("app/pages"))?;
    std::fs::create_dir_all(project_dir.join("app/components"))?;
    std::fs::create_dir_all(project_dir.join("db/migrations"))?;
    std::fs::create_dir_all(project_dir.join("config"))?;
    std::fs::create_dir_all(project_dir.join("public"))?;
    std::fs::create_dir_all(project_dir.join("dist"))?;

    // Create app/api/main.cln
    let api_main = format!(
        r#"// {name} - API Routes
// Backend API endpoints

import:
	frame.web

endpoints:
	GET /api/hello:
		return "Hello from {name}!"

	GET /api/health:
		return "OK"
"#,
        name = name
    );
    std::fs::write(project_dir.join("app/api/main.cln"), api_main)?;

    // Create app/pages/home.cln
    let home_page = format!(
        r#"// Home Page
// {name}

import:
    frame.ui

component: name="HomePage"
    layout:
        div: class="container"
            h1: "Welcome to {name}"
            p: "Built with Frame Framework"
"#,
        name = name
    );
    std::fs::write(project_dir.join("app/pages/home.cln"), home_page)?;

    // Create app/components/header.cln
    let header_component = format!(
        r#"// Header Component
// Reusable navigation header

import:
    frame.ui

component: name="Header"
    nav: class="header"
        a: href="/" "{name}"
        div: class="nav-links"
            a: href="/about" "About"
"#,
        name = name
    );
    std::fs::write(project_dir.join("app/components/header.cln"), header_component)?;

    // Create db/schema.cln
    let schema = r#"// Database Schema
// Define your data models here

import:
    frame.data

model: name="User" table="users"
    integer id
    string email
    string name
    boolean active = true
"#;
    std::fs::write(project_dir.join("db/schema.cln"), schema)?;

    // Create config/app.cln
    let config_cln = format!(
        r#"// Application Configuration
// {name}

config:
    name = "{name}"
    version = "0.1.0"
    environment = "development"

server:
    port = {port}
    host = "127.0.0.1"

database:
    driver = "sqlite"
    path = "db/{name}.db"
"#,
        name = name,
        port = port
    );
    std::fs::write(project_dir.join("config/app.cln"), config_cln)?;

    // Create frame.toml
    let frame_toml = format!(
        r#"[project]
name = "{name}"
version = "0.1.0"
description = "A Frame full-stack web application"

[server]
port = {port}
host = "127.0.0.1"
entry = "app/api/main.cln"

[database]
driver = "sqlite"
path = "db/{name}.db"

[plugins]
frame.web = "1.0.0"
frame.ui = "1.0.0"
frame.data = "1.0.0"
"#,
        name = name,
        port = port
    );
    std::fs::write(project_dir.join("frame.toml"), frame_toml)?;

    // Create public/index.html
    let index_html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{name}</title>
    <link rel="stylesheet" href="/styles.css">
</head>
<body>
    <div id="app"></div>
    <script src="/app.js"></script>
</body>
</html>
"#,
        name = name
    );
    std::fs::write(project_dir.join("public/index.html"), index_html)?;

    // Create public/styles.css
    let styles_css = r#"/* Base styles */
* {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    color: #333;
}

.container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
}

h1 {
    margin-bottom: 1rem;
}

.header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem 2rem;
    background: #f5f5f5;
}

.nav-links a {
    margin-left: 1rem;
    text-decoration: none;
    color: #333;
}
"#;
    std::fs::write(project_dir.join("public/styles.css"), styles_css)?;

    // Create .gitignore
    let gitignore = r#"# Build output
dist/
*.wasm

# Database
db/*.db

# Dependencies
node_modules/

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Logs
*.log
"#;
    std::fs::write(project_dir.join(".gitignore"), gitignore)?;

    Ok(())
}

/// Create minimal template (single file)
fn create_minimal_template(name: &str, port: u16) -> Result<()> {
    let project_dir = Path::new(name);

    // Create minimal structure
    std::fs::create_dir_all(project_dir)?;
    std::fs::create_dir_all(project_dir.join("dist"))?;

    // Create main.cln
    let main_cln = format!(
        r#"// {name} - Minimal Frame App

import:
	frame.web

endpoints:
	GET /:
		return "Hello, World!"

	GET /api/hello:
		return "Hello from {name}!"
"#,
        name = name
    );
    std::fs::write(project_dir.join("main.cln"), main_cln)?;

    // Create frame.toml
    let frame_toml = format!(
        r#"[project]
name = "{name}"
version = "0.1.0"

[server]
port = {port}
entry = "main.cln"

[plugins]
frame.web = "1.0.0"
"#,
        name = name,
        port = port
    );
    std::fs::write(project_dir.join("frame.toml"), frame_toml)?;

    // Create .gitignore
    let gitignore = r#"dist/
*.wasm
"#;
    std::fs::write(project_dir.join(".gitignore"), gitignore)?;

    Ok(())
}

/// Build a Frame project for production
pub fn build_project(input: &str, output: &str, optimize: &str) -> Result<()> {
    let config = Config::load()?;

    let input_path = Path::new(input);

    // Determine entry file
    let entry_file = if input_path.is_file() {
        input_path.to_path_buf()
    } else if input_path.is_dir() {
        // Look for frame.toml to find entry point
        let frame_toml = input_path.join("frame.toml");
        if frame_toml.exists() {
            // Parse frame.toml to find entry
            let toml_content = std::fs::read_to_string(&frame_toml)?;
            if let Some(entry) = parse_entry_from_toml(&toml_content) {
                input_path.join(entry)
            } else {
                // Default entry points
                let default_entries = [
                    "app/api/main.cln",
                    "main.cln",
                    "src/main.cln",
                ];
                default_entries
                    .iter()
                    .map(|e| input_path.join(e))
                    .find(|p| p.exists())
                    .ok_or_else(|| CleenError::FileNotFound {
                        path: "Entry file not found".to_string(),
                    })?
            }
        } else {
            // No frame.toml, try default entries
            let default_entries = [
                "app/api/main.cln",
                "main.cln",
                "src/main.cln",
            ];
            default_entries
                .iter()
                .map(|e| input_path.join(e))
                .find(|p| p.exists())
                .ok_or_else(|| CleenError::FileNotFound {
                    path: "Entry file not found".to_string(),
                })?
        }
    } else {
        return Err(CleenError::FileNotFound {
            path: input.to_string(),
        });
    };

    // Find compiler
    let cln_path = config.get_shim_path();
    if !cln_path.exists() {
        return Err(CleenError::NoActiveVersion);
    }

    // Create output directory
    let output_dir = if input_path.is_dir() {
        input_path.join(output)
    } else {
        Path::new(output).to_path_buf()
    };
    std::fs::create_dir_all(&output_dir)?;

    // Determine output file name
    let wasm_name = entry_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("app");
    let wasm_path = output_dir.join(format!("{}.wasm", wasm_name));

    println!("Building Frame project...");
    println!("   Entry: {:?}", entry_file);
    println!("   Output: {:?}", wasm_path);
    println!("   Optimization: level {}", optimize);
    println!();

    // Compile with optimization level
    let compile_output = Command::new(&cln_path)
        .args(["compile"])
        .arg(&entry_file)
        .args(["-o"])
        .arg(&wasm_path)
        .arg("--plugins")
        .args(["--opt-level", optimize])
        .output()
        .map_err(|e| CleenError::CompilationFailed {
            message: format!("Failed to run compiler: {e}"),
        })?;

    if !compile_output.status.success() {
        let stderr = String::from_utf8_lossy(&compile_output.stderr);
        println!("❌ Build failed:");
        println!("{stderr}");
        return Err(CleenError::CompilationFailed {
            message: stderr.to_string(),
        });
    }

    // Get file size
    let metadata = std::fs::metadata(&wasm_path)?;
    let size_kb = metadata.len() as f64 / 1024.0;

    println!("✅ Build successful!");
    println!();
    println!("   Output: {:?}", wasm_path);
    println!("   Size: {:.1} KB", size_kb);
    println!();
    println!("To run in production:");
    println!("   frame-runtime {:?}", wasm_path);

    Ok(())
}

/// Parse entry point from frame.toml content
fn parse_entry_from_toml(content: &str) -> Option<String> {
    // Simple parsing - look for entry = "..."
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("entry") {
            if let Some(value) = line.split('=').nth(1) {
                let value = value.trim().trim_matches('"').trim_matches('\'');
                return Some(value.to_string());
            }
        }
    }
    None
}
