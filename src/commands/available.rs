use anyhow::Result;
use crate::core::github::GitHubClient;

pub fn list_available_versions() -> Result<()> {
    println!("Clean Language Compiler Versions");
    println!("=================================");
    println!();
    
    let github_client = GitHubClient::new();
    
    match github_client.get_releases() {
        Ok(releases) => {
            if releases.is_empty() {
                println!("🛠️  No releases are available yet.");
                println!("   The Clean Language compiler may still be in development.");
                println!("   Check back later or follow the repository for updates:");
                println!("   🌐 https://github.com/Ivan-Pasco/clean-language-compiler/releases");
            } else {
                println!("📋 Available versions:");
                println!();
                
                for (i, release) in releases.iter().enumerate() {
                    let status = if i == 0 { " (latest)" } else { "" };
                    let prerelease = if release.prerelease { " [prerelease]" } else { "" };
                    
                    println!("  • {}{}{}", release.tag_name, status, prerelease);
                    if !release.name.is_empty() && release.name != release.tag_name {
                        println!("    {}", release.name);
                    }
                    
                    // Show available assets for this release
                    if !release.assets.is_empty() {
                        let asset_names: Vec<String> = release.assets.iter()
                            .map(|a| a.name.clone())
                            .collect();
                        println!("    Assets: {}", asset_names.join(", "));
                    }
                }
                
                println!();
                println!("🔧 To install a version, run:");
                println!("  cleanmanager install <version>");
                println!("  cleanmanager install latest");
                println!();
                
                println!("💡 Examples:");
                if let Some(latest) = releases.first() {
                    println!("  cleanmanager install {}", latest.tag_name);
                }
                println!("  cleanmanager install latest");
            }
        }
        Err(e) => {
            println!("⚠️  Unable to fetch releases from GitHub: {}", e);
            println!("   This may be because:");
            println!("   • The repository doesn't have releases yet");
            println!("   • Network connectivity issues");
            println!("   • GitHub API rate limiting");
            println!();
            println!("   Please check the repository manually:");
            println!("   🌐 https://github.com/Ivan-Pasco/clean-language-compiler/releases");
            println!();
            println!("🔧 Once you find a version you want, install it with:");
            println!("  cleanmanager install <version>");
            println!("  cleanmanager install latest");
        }
    }
    
    Ok(())
}