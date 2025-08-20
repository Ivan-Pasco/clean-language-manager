use anyhow::{anyhow, Result};
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Detect the current shell being used
pub fn detect_shell() -> String {
    // Check SHELL environment variable first
    if let Ok(shell_path) = env::var("SHELL") {
        if shell_path.contains("zsh") {
            return "zsh".to_string();
        } else if shell_path.contains("bash") {
            return "bash".to_string();
        } else if shell_path.contains("fish") {
            return "fish".to_string();
        }
    }

    // Fallback to bash as most common
    "bash".to_string()
}

/// Get the appropriate shell configuration file path for the current shell
pub fn get_shell_config_path() -> Result<PathBuf> {
    let home = env::var("HOME")
        .map(PathBuf::from)
        .or_else(|_| env::var("USERPROFILE").map(PathBuf::from))
        .map_err(|_| anyhow!("Could not find home directory"))?;

    let shell = detect_shell();
    match shell.as_str() {
        "zsh" => Ok(home.join(".zshrc")),
        "fish" => {
            let config_dir = home.join(".config").join("fish");
            std::fs::create_dir_all(&config_dir)?;
            Ok(config_dir.join("config.fish"))
        }
        "bash" => {
            // Prefer .bashrc, fallback to .bash_profile
            let bashrc = home.join(".bashrc");
            let bash_profile = home.join(".bash_profile");

            if bashrc.exists() {
                Ok(bashrc)
            } else {
                Ok(bash_profile)
            }
        }
        _ => Ok(home.join(".bashrc")), // Default fallback
    }
}

/// Check if a directory is already in PATH
pub fn is_in_path(bin_dir: &Path) -> bool {
    if let Ok(path) = env::var("PATH") {
        let bin_dir_str = bin_dir.to_string_lossy();
        path.split(':').any(|p| p == bin_dir_str)
    } else {
        false
    }
}

/// Add a directory to PATH in the shell configuration file
pub fn add_to_path(bin_dir: &Path) -> Result<()> {
    let shell = detect_shell();
    let config_path = get_shell_config_path()?;
    let bin_dir_str = bin_dir.to_string_lossy();

    // Check if already configured in the file
    if config_path.exists() && is_already_configured(&config_path, &bin_dir_str)? {
        println!("✅ PATH already configured in {}", config_path.display());
        return Ok(());
    }

    // Prepare the export line based on shell type
    let export_line = match shell.as_str() {
        "fish" => format!("set -gx PATH \"{bin_dir_str}\" $PATH"),
        _ => format!("export PATH=\"{bin_dir_str}:$PATH\""),
    };

    // Add to config file
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config_path)?;

    writeln!(file)?;
    writeln!(file, "# Added by Clean Language Manager")?;
    writeln!(file, "{export_line}")?;

    println!("✅ Added to PATH in {}", config_path.display());
    Ok(())
}

/// Check if the PATH export is already configured in the shell config file
fn is_already_configured(config_path: &Path, bin_dir: &str) -> Result<bool> {
    let file = File::open(config_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.contains(bin_dir) && (line.contains("export PATH") || line.contains("set -gx PATH"))
        {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Get shell-specific instructions for reloading configuration
pub fn get_reload_instructions() -> String {
    let shell = detect_shell();
    let config_path = get_shell_config_path()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "~/.bashrc".to_string());

    match shell.as_str() {
        "zsh" => format!("source {config_path}"),
        "fish" => "source ~/.config/fish/config.fish".to_string(),
        _ => format!("source {config_path}"),
    }
}
