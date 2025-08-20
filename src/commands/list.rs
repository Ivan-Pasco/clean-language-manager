use crate::core::{config::Config, version::VersionManager};
use crate::commands::update;
use crate::error::Result;

pub fn list_versions() -> Result<()> {
    let config = Config::load()?;
    let version_manager = VersionManager::new(config);
    
    let versions = version_manager.list_installed_versions()?;
    
    if versions.is_empty() {
        println!("No Clean Language versions installed.");
        println!();
        println!("To install a version, run:");
        println!("  cleanmanager install <version>");
        return Ok(());
    }
    
    println!("Installed Clean Language versions:");
    println!();
    
    for version_info in versions {
        let status = if version_info.is_active {
            "✅ (active)"
        } else if !version_info.is_valid {
            "❌ (invalid)"
        } else {
            ""
        };
        
        println!("  {} {}", version_info.version, status);
        
        if !version_info.is_valid {
            println!("    Binary not found: {:?}", version_info.binary_path);
        }
    }
    
    println!();
    
    if let Some(active_version) = version_manager.get_active_version() {
        println!("Active version: {}", active_version);
    } else {
        println!("No active version set. Use 'cleanmanager use <version>' to activate a version.");
    }
    
    let _ = update::check_updates_if_needed();
    
    Ok(())
}