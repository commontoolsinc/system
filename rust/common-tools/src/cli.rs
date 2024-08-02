use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// CLI struct implementing [clap::Parser].
#[derive(Parser)]
#[command(version, about="Common Runtime Tools", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Command to execute.
    #[command(subcommand)]
    pub command: Command,
}

/// CLI subcommands implementing [clap::Subcommand].
#[derive(Subcommand)]
pub enum Command {
    /// Executes a module.
    Run {
        /// Path to module file to execute.
        module_path: PathBuf,

        /// Connect to runtime server on provided port.
        #[arg(short, long, default_value_t = 8081)]
        port: u16,

        /// Use input from stdin as "input" to module.
        #[arg(short = 'i', long)]
        stdin: bool,
    },

    /// Start the necessary runtime and build server.
    Serve {
        /// Runtime server listens on provided port.
        #[arg(short, long, default_value_t = 8081)]
        port: u16,
    },
}
