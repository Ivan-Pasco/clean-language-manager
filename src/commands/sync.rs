use crate::commands::install;
use crate::core::{config::Config, version::VersionManager};
use crate::error::{CleanManagerError, Result};
use std::env;

pub fn sync_project_version() -> Result<()> {
    let config = Config::load()?;
    let version_manager = VersionManager::new(config.clone());

    println!("🔄 Syncing project version from .cleanlanguage/.cleanversion file");

    // Get current directory for display
    let current_dir = env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("current project");

    println!("📁 Project: {project_name}");
    println!("   Directory: {current_dir:?}");

    // Look for .cleanlanguage/.cleanversion file
    match config.get_project_version() {
        Some(project_version) => {
            println!("📋 Found .cleanlanguage/.cleanversion file specifying: {project_version}");

            // Check if version is already installed
            if version_manager.is_version_installed(&project_version) {
                println!("✅ Version {project_version} is already installed");
                println!();
                println!("🎉 Project is ready to use!");
                println!("   Run 'cln --version' to verify");
            } else {
                println!("📦 Version {project_version} is not installed, installing now...");
                println!();

                // Install the version
                match install::install_version(&project_version) {
                    Ok(_) => {
                        println!();
                        println!("🎉 Successfully synced project version!");
                        println!(
                            "   Project {project_name} is now ready to use Clean Language v{project_version}"
                        );
                        println!();
                        println!("🔍 Verify with:");
                        println!("  cleanmanager doctor");
                        println!("  cln --version");
                    }
                    Err(e) => {
                        println!("❌ Failed to install version {project_version}: {e}");
                        println!();
                        println!("💡 You can try:");
                        println!("  cleanmanager available    # Check available versions");
                        println!("  cleanmanager install {project_version}   # Install manually");
                        return Err(e);
                    }
                }
            }
        }
        None => {
            println!("❌ No .cleanlanguage/.cleanversion file found in current directory or parent directories");
            println!();
            println!("💡 To set up project-specific version management:");
            println!("  1. Install a Clean Language version:");
            println!("     cleanmanager install 0.1.2");
            println!("  2. Set it for this project:");
            println!("     cleanmanager local 0.1.2");
            println!("  3. Then you can use 'cleanmanager sync' in this project");
            println!();
            println!("🔍 Or check what versions are available:");
            println!("  cleanmanager available");

            return Err(CleanManagerError::ConfigError {
                message: "No .cleanlanguage/.cleanversion file found".to_string(),
            });
        }
    }

    Ok(())
}
