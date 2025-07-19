use crate::core::{config::Config, version::VersionManager, shim::ShimManager};
use crate::error::Result;
use std::process::Command;

pub fn check_environment() -> Result<()> {
    println!("🔍 Clean Language Manager - Environment Check");
    println!();
    
    let config = Config::load()?;
    let version_manager = VersionManager::new(config.clone());
    let shim_manager = ShimManager::new(config.clone());
    
    let mut issues_found = 0;
    
    // Check cleanmanager directories
    println!("📁 Directory Structure:");
    let cleanmanager_dir = &config.cleanmanager_dir;
    println!("  cleanmanager directory: {:?}", cleanmanager_dir);
    
    if cleanmanager_dir.exists() {
        println!("    ✅ exists");
    } else {
        println!("    ❌ missing");
        issues_found += 1;
    }
    
    let versions_dir = config.get_versions_dir();
    println!("  versions directory: {:?}", versions_dir);
    if versions_dir.exists() {
        println!("    ✅ exists");
    } else {
        println!("    ❌ missing");
        issues_found += 1;
    }
    
    let bin_dir = config.get_bin_dir();
    println!("  bin directory: {:?}", bin_dir);
    if bin_dir.exists() {
        println!("    ✅ exists");
    } else {
        println!("    ❌ missing");
        issues_found += 1;
    }
    
    println!();
    
    // Check installed versions
    println!("📦 Installed Versions:");
    let versions = version_manager.list_installed_versions()?;
    if versions.is_empty() {
        println!("  ⚠️  No versions installed");
    } else {
        for version_info in &versions {
            println!("  {} {}", 
                version_info.version,
                if version_info.is_valid { "✅" } else { "❌" }
            );
            
            if !version_info.is_valid {
                issues_found += 1;
            }
        }
    }
    
    println!();
    
    // Check active version and shim
    println!("🔗 Active Version & Shim:");
    if let Some(active_version) = version_manager.get_active_version() {
        println!("  active version: {}", active_version);
        
        let shim_path = config.get_shim_path();
        println!("  shim path: {:?}", shim_path);
        
        if shim_path.exists() {
            println!("    ✅ shim exists");
            
            // Verify shim works
            if shim_manager.verify_shim()? {
                println!("    ✅ shim is valid");
            } else {
                println!("    ❌ shim is invalid");
                issues_found += 1;
            }
        } else {
            println!("    ❌ shim missing");
            issues_found += 1;
        }
    } else {
        println!("  ⚠️  No active version set");
    }
    
    println!();
    
    // Check PATH configuration
    println!("🛣️  PATH Configuration:");
    let bin_dir_str = bin_dir.to_string_lossy();
    
    if let Ok(path) = std::env::var("PATH") {
        if path.contains(&*bin_dir_str) {
            println!("  ✅ cleanmanager bin directory is in PATH");
        } else {
            println!("  ❌ cleanmanager bin directory NOT in PATH");
            println!("    Add this to your shell config:");
            println!("    export PATH=\"{}:$PATH\"", bin_dir_str);
            issues_found += 1;
        }
    } else {
        println!("  ❌ PATH environment variable not found");
        issues_found += 1;
    }
    
    // Try to run cln command
    println!();
    println!("🧪 Command Test:");
    match Command::new("cln").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version_output = String::from_utf8_lossy(&output.stdout);
                println!("  ✅ 'cln --version' works: {}", version_output.trim());
            } else {
                println!("  ❌ 'cln --version' failed");
                issues_found += 1;
            }
        }
        Err(_) => {
            println!("  ❌ 'cln' command not found");
            issues_found += 1;
        }
    }
    
    println!();
    
    // Summary
    if issues_found == 0 {
        println!("🎉 Environment looks good! No issues found.");
    } else {
        println!("⚠️  Found {} issue(s) that need attention.", issues_found);
        println!();
        println!("💡 To fix issues:");
        println!("  - Run 'cleanmanager init' to set up shell configuration");
        println!("  - Run 'cleanmanager install <version>' to install a version");
        println!("  - Run 'cleanmanager use <version>' to activate a version");
    }
    
    Ok(())
}