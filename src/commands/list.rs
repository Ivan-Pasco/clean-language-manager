use crate::commands::update;
use crate::core::{config::Config, frame, version::VersionManager};
use crate::error::Result;

pub fn list_versions(show_frame: bool) -> Result<()> {
    let config = Config::load()?;

    if show_frame {
        // List Frame CLI versions only
        let frame_versions = frame::list_frame_versions(&config)?;

        if frame_versions.is_empty() {
            println!("No Frame CLI versions installed");
            println!();
            println!("To install Frame CLI:");
            println!("   cleen frame install");
        } else {
            println!("Installed Frame CLI versions:");
            for v in &frame_versions {
                let marker = if config.frame_version.as_deref() == Some(v) {
                    "  ✅ "
                } else {
                    "     "
                };
                println!("{marker}{v}");
            }

            if let Some(active) = &config.frame_version {
                println!();
                println!("Active Frame CLI version: {active}");
            }
        }

        return Ok(());
    }

    // List compiler versions
    let version_manager = VersionManager::new(config.clone());
    let versions = version_manager.list_installed_versions()?;

    if versions.is_empty() {
        println!("No Clean Language versions installed.");
        println!();
        println!("To install a version, run:");
        println!("  cleen install <version>");
        return Ok(());
    }

    println!("Installed Clean Language Compiler versions:");
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
        println!("Active compiler version: {active_version}");
    } else {
        println!("No active version set. Use 'cleen use <version>' to activate a version.");
    }

    // Also show Frame CLI info
    let frame_versions = frame::list_frame_versions(&config)?;
    if !frame_versions.is_empty() {
        println!();
        println!("Installed Frame CLI versions:");
        for v in &frame_versions {
            let marker = if config.frame_version.as_deref() == Some(v) {
                "  ✅ "
            } else {
                "     "
            };
            let compat_marker = if let Some(compiler_version) = &config.active_version {
                use crate::core::compatibility;
                if compatibility::check_frame_compatibility(compiler_version, v).is_ok() {
                    "(compatible)"
                } else {
                    "(⚠️  incompatible with active compiler)"
                }
            } else {
                ""
            };
            println!("{marker}{v} {compat_marker}");
        }
    }

    let _ = update::check_updates_if_needed();

    Ok(())
}
