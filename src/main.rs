use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod core;
mod error;
mod utils;

#[derive(Parser)]
#[clap(name = "cleen")]
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
        version: String,
        /// Also install Frame CLI
        #[clap(long)]
        with_frame: bool,
        /// Skip Frame CLI prompt
        #[clap(long)]
        no_frame: bool,
    },
    /// Install the version specified in .cleanlanguage/.cleanversion file
    Sync,
    /// List installed versions
    List {
        /// List Frame CLI versions
        #[clap(long)]
        frame: bool,
    },
    /// List available versions from GitHub
    Available,
    /// Switch to a specific version globally
    Use {
        /// Version to use globally
        version: String,
        /// Use Frame CLI version instead
        #[clap(long)]
        frame: bool,
    },
    /// Set project-specific version (creates .cleanlanguage/.cleanversion file)
    Local {
        /// Version to use in this project
        version: String,
    },
    /// Uninstall a specific version
    Uninstall {
        /// Version to uninstall
        version: String,
        /// Uninstall Frame CLI version instead
        #[clap(long)]
        frame: bool,
        /// Force uninstall even if Frame depends on it
        #[clap(long)]
        force: bool,
    },
    /// Initialize shell configuration
    Init,
    /// Check and repair environment setup
    Doctor {
        /// Check Frame CLI installation
        #[clap(long)]
        frame: bool,
    },
    /// Check for Clean Language compiler updates
    Update,
    /// Update cleen itself to the latest version
    SelfUpdate,
    /// Frame CLI management
    Frame {
        #[clap(subcommand)]
        command: FrameCommands,
    },
}

#[derive(Subcommand)]
enum FrameCommands {
    /// Install Frame CLI
    Install {
        /// Version to install (optional, auto-detects compatible version)
        version: Option<String>,
    },
    /// List installed Frame CLI versions
    List,
    /// Switch to a specific Frame CLI version
    Use {
        /// Version to use
        version: String,
    },
    /// Uninstall a Frame CLI version
    Uninstall {
        /// Version to uninstall
        version: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install {
            version,
            with_frame,
            no_frame,
        } => commands::install::install_version(&version, with_frame, no_frame)
            .map_err(|e| anyhow::anyhow!(e)),
        Commands::Sync => commands::sync::sync_project_version().map_err(|e| anyhow::anyhow!(e)),
        Commands::List { frame } => {
            commands::list::list_versions(frame).map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Available => commands::available::list_available_versions(),
        Commands::Use { version, frame } => {
            commands::use_version::use_version(&version, frame).map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Local { version } => {
            commands::local::set_local_version(&version).map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Uninstall {
            version,
            frame,
            force,
        } => commands::uninstall::uninstall_version(&version, frame, force)
            .map_err(|e| anyhow::anyhow!(e)),
        Commands::Init => commands::init::init_shell().map_err(|e| anyhow::anyhow!(e)),
        Commands::Doctor { frame } => {
            commands::doctor::check_environment(frame).map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Update => commands::update::check_for_updates().map_err(|e| anyhow::anyhow!(e)),
        Commands::SelfUpdate => {
            commands::update::update_self_auto().map_err(|e| anyhow::anyhow!(e))
        }
        Commands::Frame { command } => match command {
            FrameCommands::Install { version } => {
                core::frame::install_frame(version.as_deref(), false)
                    .map_err(|e| anyhow::anyhow!(e))
            }
            FrameCommands::List => {
                let config = core::config::Config::load().map_err(|e| anyhow::anyhow!(e))?;
                let versions = core::frame::list_frame_versions(&config)
                    .map_err(|e| anyhow::anyhow!(e))?;

                if versions.is_empty() {
                    println!("No Frame CLI versions installed");
                    println!();
                    println!("To install Frame CLI:");
                    println!("   cleen frame install");
                } else {
                    println!("Installed Frame CLI versions:");
                    for v in &versions {
                        let marker = if config.frame_version.as_deref() == Some(v) {
                            "* "
                        } else {
                            "  "
                        };
                        println!("{marker}{v}");
                    }
                }
                Ok(())
            }
            FrameCommands::Use { version } => {
                core::frame::use_frame_version(&version).map_err(|e| anyhow::anyhow!(e))
            }
            FrameCommands::Uninstall { version } => {
                core::frame::uninstall_frame_version(&version).map_err(|e| anyhow::anyhow!(e))
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}
