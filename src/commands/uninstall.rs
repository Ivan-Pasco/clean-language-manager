use crate::core::config::Config;
use crate::error::{CleanManagerError, Result};
use std::io::{self, Write};

pub fn uninstall_version(version: &str) -> Result<()> {
    println!("Uninstalling Clean Language version: {version}");

    let mut config = Config::load()?;
    let version_dir = config.get_version_dir(version);

    // Check if version exists
    if !version_dir.exists() {
        return Err(CleanManagerError::VersionNotFound {
            version: version.to_string(),
        });
    }

    // Check if this is the currently active version
    if let Some(ref active_version) = config.active_version {
        if active_version == version {
            println!("⚠️  Version {version} is currently active.");
            print!("Do you want to continue uninstalling it? [y/N]: ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            if !input.trim().to_lowercase().starts_with('y') {
                println!("Uninstall cancelled.");
                return Ok(());
            }

            // Clear the active version since we're uninstalling it
            config.clear_active_version()?;
            println!("Cleared active version setting.");
        }
    }

    // Confirm uninstallation
    print!(
        "Are you sure you want to uninstall version {version}? [y/N]: "
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    if !input.trim().to_lowercase().starts_with('y') {
        println!("Uninstall cancelled.");
        return Ok(());
    }

    // Remove the version directory
    println!("Removing version directory: {version_dir:?}");
    std::fs::remove_dir_all(&version_dir)?;

    println!(
        "✅ Successfully uninstalled Clean Language version {version}"
    );

    // Show remaining versions if any
    let versions_dir = config.get_versions_dir();
    if versions_dir.exists() {
        let remaining_versions: Vec<_> = std::fs::read_dir(&versions_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.path().is_dir() {
                        e.file_name().to_str().map(|s| s.to_string())
                    } else {
                        None
                    }
                })
            })
            .collect();

        if remaining_versions.is_empty() {
            println!("No versions remaining.");
        } else {
            println!("\nRemaining installed versions:");
            for v in remaining_versions {
                println!("  • {v}");
            }
        }
    }

    Ok(())
}
