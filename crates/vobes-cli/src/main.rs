//! vbs — Vobes CLI.
//!
//! Natural extension of the desktop app. Same core, two faces.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

mod commands;
mod output;

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
    List {
        /// Emit raw JSON instead of a table (for AI agents / scripting).
        #[clap(long)]
        json: bool,
    },
    /// Inspect one vobe in detail.
    Show {
        /// Vobe name or path.
        name: String,
        /// Emit raw JSON instead of a formatted view (for AI agents / scripting).
        #[clap(long)]
        json: bool,
    },
    /// Show activity timeline.
    Log {
        /// Limit number of events.
        #[clap(long, default_value = "20")]
        limit: usize,
        /// Emit raw JSON instead of a formatted view (for AI agents / scripting).
        #[clap(long)]
        json: bool,
    },
    /// Dump a compact context pack (full vobe + recent activity) as JSON.
    Context {
        /// Vobe name or path.
        name: String,
    },
    /// Stream activity as newline-delimited JSON (NDJSON). Ctrl-C to stop.
    Watch,
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
        Some(cmd) => match vobes_cli::app::App::load() {
            Ok(app) => match dispatch(&app, cmd) {
                Ok(()) => std::process::ExitCode::SUCCESS,
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::ExitCode::FAILURE
                }
            },
            Err(e) => {
                eprintln!("error: {e}");
                std::process::ExitCode::FAILURE
            }
        },
    }
}

fn dispatch(app: &vobes_cli::app::App, cmd: Command) -> vobes_core::Result<()> {
    match cmd {
        Command::Scan => commands::scan::run(app),
        Command::List { json } => commands::list::run(app, json),
        Command::Show { name, json } => commands::show::run(app, &name, json),
        Command::Log { limit, json } => commands::log::run(app, limit, json),
        Command::Context { name } => commands::context::run(app, &name),
        Command::Watch => commands::watch::run(app),
        Command::Sync => commands::sync::run(app),
        Command::Add { path } => commands::add::run(app, &path),
        Command::Rm { name } => commands::rm::run(app, &name),
        Command::Open { name } => commands::open::run(app, &name),
        Command::Export { out } => commands::export::run(app, out.as_deref()),
        Command::Init => commands::init::run(app),
    }
}
