use crate::core::{compatibility, config::Config, frame};
use crate::error::{CleenError, Result};
use dialoguer::Confirm;
use std::io::{self, Write};

pub fn uninstall_version(version: &str, is_frame: bool, force: bool) -> Result<()> {
    if is_frame {
        // Uninstall Frame CLI
        return frame::uninstall_frame_version(version);
    }

    // Uninstall compiler version
    println!("Uninstalling Clean Language version: {version}");

    let mut config = Config::load()?;
    let version_dir = config.get_version_dir(version);

    // Check if version exists
    if !version_dir.exists() {
        return Err(CleenError::VersionNotFound {
            version: version.to_string(),
        });
    }

    // Check if Frame depends on this compiler version
    if !force {
        if let Some(frame_version) = &config.frame_version {
            let compat_matrix = compatibility::CompatibilityMatrix::new();
            if let Some(required_compiler) = compat_matrix.get_required_compiler_version(frame_version) {
                // Check if this compiler version is required for the installed Frame
                if compatibility::is_version_gte(version, &required_compiler) {
                    // Frame might depend on this compiler
                    println!("⚠️  Frame CLI {} may depend on this compiler version", frame_version);
                    println!("   Uninstalling may cause Frame CLI to stop working.");
                    println!();

                    match Confirm::new()
                        .with_prompt("Do you want to continue anyway?")
                        .default(false)
                        .interact()
                    {
                        Ok(true) => {
                            // Continue with uninstall
                        }
                        _ => {
                            println!("Uninstall cancelled.");
                            println!();
                            println!("To force uninstall: cleen uninstall {} --force", version);
                            return Ok(());
                        }
                    }
                }
            }
        }
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
    print!("Are you sure you want to uninstall version {version}? [y/N]: ");
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

    println!("✅ Successfully uninstalled Clean Language version {version}");

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
