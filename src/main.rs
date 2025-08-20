use clap::{Parser, Subcommand};
use anyhow::Result;

mod commands;
mod core;
mod error;
mod utils;

#[derive(Parser)]
#[clap(name = "cleanmanager")]
#[clap(about = "Clean Language version manager")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a specific version of Clean Language
    Install { 
        /// Version to install (e.g., 1.2.3, latest)
        version: String 
    },
    /// Install the version specified in .cleanlanguage/.cleanversion file
    Sync,
    /// List installed versions
    List,
    /// List available versions from GitHub
    Available,
    /// Switch to a specific version globally
    Use { 
        /// Version to use globally
        version: String 
    },
    /// Set project-specific version (creates .cleanlanguage/.cleanversion file)
    Local {
        /// Version to use in this project
        version: String
    },
    /// Uninstall a specific version
    Uninstall { 
        /// Version to uninstall
        version: String 
    },
    /// Initialize shell configuration
    Init,
    /// Check and repair environment setup
    Doctor,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install { version } => {
            commands::install::install_version(&version).map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Sync => {
            commands::sync::sync_project_version().map_err(|e| anyhow::anyhow!(e))
        }
        Commands::List => {
            commands::list::list_versions().map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Available => {
            commands::available::list_available_versions()
        }
        Commands::Use { version } => {
            commands::use_version::use_version(&version).map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Local { version } => {
            commands::local::set_local_version(&version).map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Uninstall { version } => {
            commands::uninstall::uninstall_version(&version).map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Init => {
            commands::init::init_shell().map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Doctor => {
            commands::doctor::check_environment().map_err(|e| anyhow::anyhow!(e))
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
