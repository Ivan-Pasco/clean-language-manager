use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod core;
mod error;
mod plugin;
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
    /// Clean up old compiler and plugin versions
    Cleanup {
        /// Actually remove versions (without this flag, shows what would be removed)
        #[clap(long)]
        confirm: bool,
        /// Number of old versions to keep (default: 3)
        #[clap(long, default_value = "3")]
        keep: usize,
        /// Clean up plugins instead of compiler versions
        #[clap(long)]
        plugins: bool,
    },
    /// Frame CLI management
    Frame {
        #[clap(subcommand)]
        command: FrameCommands,
    },
    /// Plugin management
    Plugin {
        #[clap(subcommand)]
        command: PluginCommands,
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

#[derive(Subcommand)]
enum PluginCommands {
    /// Install a plugin from the registry
    Install {
        /// Plugin name or name@version (e.g., frame.web or frame.web@1.0.0)
        plugin: String,
        /// Install from a local directory instead of registry
        #[clap(long)]
        local: bool,
    },
    /// List installed plugins
    List,
    /// Create a new plugin project
    Create {
        /// Name of the plugin to create
        name: String,
    },
    /// Build the plugin in the current directory
    Build,
    /// Publish the plugin to the registry
    Publish,
    /// Remove an installed plugin
    Remove {
        /// Name of the plugin to remove
        name: String,
    },
    /// Switch to a specific plugin version
    Use {
        /// Plugin name
        name: String,
        /// Version to use
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
        Commands::Cleanup {
            confirm,
            keep,
            plugins,
        } => {
            if plugins {
                if confirm {
                    commands::cleanup::cleanup_plugins_execute().map_err(|e| anyhow::anyhow!(e))
                } else {
                    commands::cleanup::cleanup_plugins_dry_run().map_err(|e| anyhow::anyhow!(e))
                }
            } else if confirm {
                commands::cleanup::cleanup_execute(keep).map_err(|e| anyhow::anyhow!(e))
            } else {
                commands::cleanup::cleanup_dry_run(keep).map_err(|e| anyhow::anyhow!(e))
            }
        }
        Commands::Frame { command } => match command {
            FrameCommands::Install { version } => {
                core::frame::install_frame(version.as_deref(), false)
                    .map_err(|e| anyhow::anyhow!(e))
            }
            FrameCommands::List => {
                let config = core::config::Config::load().map_err(|e| anyhow::anyhow!(e))?;
                let versions =
                    core::frame::list_frame_versions(&config).map_err(|e| anyhow::anyhow!(e))?;

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
        Commands::Plugin { command } => match command {
            PluginCommands::Install { plugin, local } => {
                if local {
                    let path = std::path::Path::new(&plugin);
                    commands::plugin::install_local_plugin(path).map_err(|e| anyhow::anyhow!(e))
                } else {
                    commands::plugin::install_plugin(&plugin).map_err(|e| anyhow::anyhow!(e))
                }
            }
            PluginCommands::List => {
                commands::plugin::list_plugins().map_err(|e| anyhow::anyhow!(e))
            }
            PluginCommands::Create { name } => {
                commands::plugin::create_plugin(&name).map_err(|e| anyhow::anyhow!(e))
            }
            PluginCommands::Build => {
                commands::plugin::build_plugin().map_err(|e| anyhow::anyhow!(e))
            }
            PluginCommands::Publish => {
                commands::plugin::publish_plugin().map_err(|e| anyhow::anyhow!(e))
            }
            PluginCommands::Remove { name } => {
                commands::plugin::remove_plugin_command(&name).map_err(|e| anyhow::anyhow!(e))
            }
            PluginCommands::Use { name, version } => {
                commands::plugin::use_plugin_version(&name, &version)
                    .map_err(|e| anyhow::anyhow!(e))
            }
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}
