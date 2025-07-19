use clap::{Parser, Subcommand};

mod commands;
mod core;
mod utils;
mod error;

use error::Result;

#[derive(Parser)]
#[clap(name = "cleanmanager")]
#[clap(about = "Version manager for Clean Language compiler")]
#[clap(version = "0.1.0")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install a specific version of Clean Language
    Install {
        /// Version to install (e.g., "1.2.3" or "latest")
        version: String,
    },
    /// List installed versions
    List,
    /// List available versions from GitHub
    Available,
    /// Switch to a specific version
    Use {
        /// Version to activate
        version: String,
    },
    /// Uninstall a specific version
    Uninstall {
        /// Version to uninstall
        version: String,
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
            println!("Installing Clean Language version: {}", version);
            // TODO: Implement install command
            Ok(())
        }
        Commands::List => {
            commands::list::list_versions()
        }
        Commands::Available => {
            println!("Fetching available versions...");
            // TODO: Implement available command
            Ok(())
        }
        Commands::Use { version } => {
            commands::use_version::use_version(&version)
        }
        Commands::Uninstall { version } => {
            println!("Uninstalling version: {}", version);
            // TODO: Implement uninstall command
            Ok(())
        }
        Commands::Init => {
            commands::init::init_shell()
        }
        Commands::Doctor => {
            commands::doctor::check_environment()
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
