use crate::core::config::Config;
use crate::error::Result;
use crate::utils::shell;
use std::io::{self, Write};

pub fn init_shell() -> Result<()> {
    println!("🔧 Initializing Clean Language Manager");
    println!();
    
    let config = Config::load()?;
    let bin_dir = config.get_bin_dir();
    let bin_dir_str = bin_dir.to_string_lossy();
    
    println!("📁 Clean Language Manager directories:");
    println!("  - Manager directory: {:?}", config.cleanmanager_dir);
    println!("  - Binary directory: {:?}", bin_dir);
    println!("  - Versions directory: {:?}", config.get_versions_dir());
    println!();
    
    // Check if PATH already contains our bin directory
    if shell::is_in_path(&bin_dir) {
        println!("✅ PATH is already configured correctly!");
        println!();
        println!("Clean Language Manager is ready to use.");
        println!("Run 'cleanmanager doctor' to verify your setup.");
        return Ok(());
    }
    
    println!("🛣️  Configuring PATH for Clean Language Manager");
    println!();
    
    let shell_name = shell::detect_shell();
    let config_path = shell::get_shell_config_path()?;
    
    println!("Detected shell: {}", shell_name);
    println!("Configuration file: {}", config_path.display());
    println!();
    
    // Ask for user consent for automatic configuration
    print!("Would you like to automatically add Clean Language Manager to your PATH? (Y/n): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    if input.is_empty() || input == "y" || input == "yes" {
        // Automatic configuration
        match shell::add_to_path(&bin_dir) {
            Ok(()) => {
                println!();
                println!("✅ Successfully configured PATH!");
                println!();
                println!("🔄 To apply the changes:");
                println!("  1. Restart your terminal, OR");
                println!("  2. Run: {}", shell::get_reload_instructions());
                println!();
                println!("Then run 'cleanmanager doctor' to verify your setup.");
            }
            Err(e) => {
                println!("❌ Automatic configuration failed: {}", e);
                println!();
                show_manual_instructions(&bin_dir_str, &shell_name, &config_path.display().to_string());
            }
        }
    } else {
        // Manual configuration requested
        println!();
        println!("📝 Manual configuration:");
        show_manual_instructions(&bin_dir_str, &shell_name, &config_path.display().to_string());
    }
    
    Ok(())
}

fn show_manual_instructions(bin_dir: &str, shell: &str, config_file: &str) {
    println!("Add the following line to your shell configuration file:");
    println!();
    
    let export_line = match shell {
        "fish" => format!("set -gx PATH \"{}\" $PATH", bin_dir),
        _ => format!("export PATH=\"{}:$PATH\"", bin_dir),
    };
    
    println!("  {}", export_line);
    println!();
    println!("Configuration file: {}", config_file);
    println!();
    println!("📝 Steps:");
    println!("  1. Add the export line above to your shell config file");
    println!("  2. Restart your terminal or run: {}", shell::get_reload_instructions());
    println!("  3. Run 'cleanmanager doctor' to verify setup");
    println!("  4. Install a Clean Language version: cleanmanager install <version>");
    println!();
    println!("💡 Tip: You can also temporarily add to PATH by running:");
    println!("  {}", export_line);
}