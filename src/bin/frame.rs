//! Standalone `frame` CLI for Clean Framework
//!
//! This is an alias for `cleen frame` commands, providing a shorter syntax:
//!   frame build    → cleen frame build
//!   frame scan     → cleen frame scan
//!   frame new      → cleen frame new
//!   frame serve    → cleen frame serve

use anyhow::Result;
use clap::{Parser, Subcommand};

// Use the cleen library
use cleen::core::frame;

#[derive(Parser)]
#[clap(name = "frame")]
#[clap(about = "Clean Framework CLI - Build full-stack web applications")]
#[clap(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Frame project
    New {
        /// Name of the project to create
        name: String,
        /// Project template: api, web, or minimal (default: api)
        #[clap(short, long, default_value = "api")]
        template: String,
        /// Port for development server (default: 3000)
        #[clap(short, long, default_value = "3000")]
        port: u16,
    },
    /// Scan and discover project files (dry-run for build)
    Scan {
        /// Project directory to scan (default: current directory)
        #[clap(default_value = ".")]
        project: String,
        /// Output format: text or json
        #[clap(short, long, default_value = "text")]
        format: String,
        /// Show verbose output including file paths
        #[clap(short, long)]
        verbose: bool,
    },
    /// Build a Frame project for production
    Build {
        /// Input file or project directory (default: current directory)
        #[clap(default_value = ".")]
        input: String,
        /// Output directory (default: dist/)
        #[clap(short, long, default_value = "dist")]
        output: String,
        /// Optimization level: 0, 1, 2, 3, s, z (default: 2)
        #[clap(short = 'O', long, default_value = "2")]
        optimize: String,
    },
    /// Start a development server for a Frame application
    Serve {
        /// Input file to serve (.cln source file with endpoints)
        #[clap(default_value = "app/api/main.cln")]
        input: String,
        /// Port to listen on (default: 3000)
        #[clap(short, long, default_value = "3000")]
        port: u16,
        /// Host to bind to (default: 127.0.0.1)
        #[clap(long, default_value = "127.0.0.1")]
        host: String,
        /// Enable debug output
        #[clap(short, long)]
        debug: bool,
    },
    /// Stop a running Frame development server
    Stop,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::New {
            name,
            template,
            port,
        } => frame::create_project(&name, &template, port).map_err(|e| anyhow::anyhow!(e)),
        Commands::Scan {
            project,
            format,
            verbose,
        } => frame::scan_project(&project, &format, verbose).map_err(|e| anyhow::anyhow!(e)),
        Commands::Build {
            input,
            output,
            optimize,
        } => frame::build_project(&input, &output, &optimize).map_err(|e| anyhow::anyhow!(e)),
        Commands::Serve {
            input,
            port,
            host,
            debug,
        } => frame::serve_application(&input, port, &host, debug).map_err(|e| anyhow::anyhow!(e)),
        Commands::Stop => frame::stop_server().map_err(|e| anyhow::anyhow!(e)),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }

    Ok(())
}
