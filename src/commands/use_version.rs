use crate::core::{config::Config, version::VersionManager, shim::ShimManager};
use crate::error::{Result, CleanManagerError};

pub fn use_version(version: &str) -> Result<()> {
    let mut config = Config::load()?;
    let version_manager = VersionManager::new(config.clone());
    
    // Validate version format
    version_manager.validate_version(version)?;
    
    // Check if version is installed
    if !version_manager.is_version_installed(version) {
        return Err(CleanManagerError::VersionNotFound {
            version: version.to_string(),
        });
    }
    
    // Update active version in config
    config.set_active_version(version.to_string())?;
    
    // Create/update shim
    let shim_manager = ShimManager::new(config);
    shim_manager.create_shim(version)?;
    
    println!("Now using Clean Language version {}", version);
    println!();
    println!("Verify with: cln --version");
    
    Ok(())
}