use crate::core::config::Config;
use crate::error::Result;
use crate::utils::shell;

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
    if let Ok(path) = std::env::var("PATH") {
        if path.contains(&*bin_dir_str) {
            println!("✅ PATH is already configured correctly!");
            println!();
            println!("Clean Language Manager is ready to use.");
            println!("Run 'cleanmanager doctor' to verify your setup.");
            return Ok(());
        }
    }
    
    println!("🛣️  Adding Clean Language Manager to PATH");
    println!();
    println!("Add the following line to your shell configuration file:");
    println!();
    
    let shell = shell::detect_shell();
    let config_file = match shell.as_str() {
        "zsh" => "~/.zshrc",
        "bash" => "~/.bashrc or ~/.bash_profile",
        "fish" => "~/.config/fish/config.fish",
        _ => "your shell configuration file",
    };
    
    println!("  export PATH=\"{}:$PATH\"", bin_dir_str);
    println!();
    println!("Configuration file: {}", config_file);
    println!();
    
    println!("📝 Manual setup steps:");
    println!("  1. Add the export line above to your shell config file");
    println!("  2. Restart your terminal or run: source {}", config_file);
    println!("  3. Run 'cleanmanager doctor' to verify setup");
    println!("  4. Install a Clean Language version: cleanmanager install <version>");
    println!();
    
    println!("💡 Tip: You can also temporarily add to PATH by running:");
    println!("  export PATH=\"{}:$PATH\"", bin_dir_str);
    
    Ok(())
}