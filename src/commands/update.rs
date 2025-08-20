use crate::core::{config::Config, github::GitHubClient};
use crate::error::Result;
use std::env;

pub fn update_self() -> Result<()> {
    println!("🔄 Checking for cleanmanager updates...");

    let github = GitHubClient::new(None);
    let releases = github.get_releases("Ivan-Pasco", "clean-language-manager")?;

    if releases.is_empty() {
        println!("❌ No releases found for cleanmanager");
        return Ok(());
    }

    let latest_release = &releases[0];
    let current_version = env!("CARGO_PKG_VERSION");

    if latest_release.tag_name.trim_start_matches('v') == current_version {
        println!("✅ cleanmanager is up to date (version {current_version})");

        let mut config = Config::load()?;
        config.update_last_self_check_time()?;

        return Ok(());
    }

    println!(
        "🎉 New version available: {} (current: {})",
        latest_release.tag_name, current_version
    );
    println!();
    println!("To update cleanmanager:");
    println!("  1. Visit: https://github.com/Ivan-Pasco/clean-language-manager/releases/latest");
    println!("  2. Or use the install script:");

    if cfg!(windows) {
        println!("     iwr https://github.com/Ivan-Pasco/clean-language-manager/releases/latest/download/install.ps1 | iex");
    } else {
        println!("     curl -sSL https://github.com/Ivan-Pasco/clean-language-manager/releases/latest/download/install.sh | bash");
    }

    println!();
    println!("  3. Or build from source:");
    println!("     git pull && cargo install --path .");

    let mut config = Config::load()?;
    config.update_last_self_check_time()?;

    Ok(())
}

pub fn check_for_updates() -> Result<()> {
    println!("🔄 Checking for Clean Language compiler updates...");

    let github = GitHubClient::new(None);
    let releases = github.get_releases("Ivan-Pasco", "clean-language-compiler")?;

    if releases.is_empty() {
        println!("❌ No releases found");
        return Ok(());
    }

    let config = Config::load()?;
    let latest_release = &releases[0];

    match &config.active_version {
        Some(current_version) => {
            if current_version == &latest_release.tag_name || current_version == "latest" {
                println!(
                    "✅ You're using the latest version: {}",
                    latest_release.tag_name
                );
            } else {
                println!(
                    "🎉 New version available: {} (current: {})",
                    latest_release.tag_name, current_version
                );
                println!();
                println!("To update:");
                println!("  cleanmanager install latest");
                println!("  cleanmanager use latest");
            }
        }
        None => {
            println!("⚠️  No version currently active");
            println!("Latest available: {}", latest_release.tag_name);
            println!();
            println!("To install:");
            println!("  cleanmanager install latest");
            println!("  cleanmanager use latest");
        }
    }

    let mut config = Config::load()?;
    config.update_last_check_time()?;

    Ok(())
}

pub fn check_updates_if_needed() -> Result<()> {
    let mut config = Config::load()?;

    if config.should_check_updates() && check_for_updates().is_ok() {
        let _ = config.update_last_check_time();
    }

    if config.should_check_self_updates() && update_self().is_ok() {
        let _ = config.update_last_self_check_time();
    }

    Ok(())
}
