//! vbs — Vobes CLI.
//!
//! Natural extension of the desktop app. Same core, two faces.
//! Command implementations arrive in Phase 7.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

use clap::{Parser, Subcommand};

/// Vobes CLI — `vbs`.
#[derive(Debug, Parser)]
#[clap(
    name = "vbs",
    version,
    about = "Vobes — developer command center",
    long_about = "Vobes unifies fragmented developer context (git, activity, project metadata) into one place."
)]
pub struct Cli {
    /// Subcommand to run.
    #[clap(subcommand)]
    pub command: Option<Command>,
}

/// All `vbs` subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Discover projects in configured roots.
    Scan,
    /// List all tracked vobes.
    List,
    /// Inspect one vobe in detail.
    Show {
        /// Vobe name or path.
        name: String,
    },
    /// Show activity timeline.
    Log {
        /// Limit number of events.
        #[clap(long, default_value = "20")]
        limit: usize,
    },
    /// Re-scan, refresh git cache, record activity.
    Sync,
    /// Manually add a vobe for a path.
    Add {
        /// Absolute or relative path to the project.
        path: String,
    },
    /// Remove a vobe from tracking.
    Rm {
        /// Vobe name.
        name: String,
    },
    /// Record an Opened event and launch editor.
    Open {
        /// Vobe name.
        name: String,
    },
    /// Export all data as JSON.
    Export {
        /// Optional custom output path. Defaults to config export path.
        #[clap(long)]
        out: Option<String>,
    },
    /// Create a default `vobes.toml` in the current directory.
    Init,
}

fn main() -> std::process::ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    match cli.command {
        None => {
            println!("vbs — Vobes CLI");
            println!();
            println!("Run `vbs --help` to see commands.");
            std::process::ExitCode::SUCCESS
        }
        Some(cmd) => {
            eprintln!("not implemented yet: {cmd:?}");
            std::process::ExitCode::FAILURE
        }
    }
}
