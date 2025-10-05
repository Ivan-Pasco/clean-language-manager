use crate::core::{
    config::Config,
    shim::ShimManager,
    version::{normalize, VersionManager},
};
use crate::error::{CleenError, Result};

pub fn use_version(version: &str) -> Result<()> {
    let mut config = Config::load()?;
    let version_manager = VersionManager::new(config.clone());

    // Normalize the version to clean format
    let clean_version = normalize::to_clean_version(version);

    // Validate version format
    version_manager.validate_version(&clean_version)?;

    // Check if version is installed (using clean version)
    if !version_manager.is_version_installed(&clean_version) {
        return Err(CleenError::VersionNotFound {
            version: clean_version.clone(),
        });
    }

    // Update active version in config (using clean version)
    config.set_active_version(clean_version.clone())?;

    // Create/update shim
    let shim_manager = ShimManager::new(config);
    shim_manager.create_shim(&clean_version)?;

    println!("✅ Activated Clean Language version {clean_version}");
    println!("Now using Clean Language version {clean_version}");
    println!();
    println!("Verify with: cln --version");

    Ok(())
}
