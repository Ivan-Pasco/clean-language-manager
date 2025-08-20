use crate::core::{config::Config, version::VersionManager};
use crate::error::{CleanManagerError, Result};
use std::env;

pub fn set_local_version(version: &str) -> Result<()> {
    let config = Config::load()?;
    let version_manager = VersionManager::new(config.clone());

    // Validate version format
    version_manager.validate_version(version)?;

    // Check if version is installed
    if !version_manager.is_version_installed(version) {
        return Err(CleanManagerError::VersionNotFound {
            version: version.to_string(),
        });
    }

    // Get current directory for display
    let current_dir = env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("current project");

    println!("Setting Clean Language version for {project_name}: {version}");

    // Create .cleanlanguage/.cleanversion file
    config.set_project_version(version)?;

    println!();
    println!("💡 Usage:");
    println!("  - When you run 'cln' in this directory, it will use version {version}");
    println!("  - The .cleanlanguage/.cleanversion file has been added to your project");
    println!("  - Consider adding .cleanlanguage/ to your version control system");
    println!();
    println!("🔍 To verify, run:");
    println!("  cleanmanager doctor");

    Ok(())
}
