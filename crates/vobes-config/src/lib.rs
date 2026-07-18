//! Vobes configuration loading and defaults.
//!
//! Config is TOML, human-readable. Every field has a default — an empty
//! `[[]]` config is valid.

#![forbid(unsafe_code)]
#![warn(missing_docs, rust_2018_idioms, clippy::all)]

mod model;
mod paths;

pub use model::*;
pub use paths::config_path;

pub use paths::{db_path, snapshots_dir, state_dir};
