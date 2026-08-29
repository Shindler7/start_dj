//! Application constants and configuration file paths.

use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

// Name of the configuration file containing startup parameters.
const TOML_NAME: &str = "start.toml";

/// Returns the full path to the configuration file (`start.toml`).
///
/// Resolves the path relative to the current working directory.
pub(crate) fn toml_path() -> Result<PathBuf> {
    let toml = PathBuf::new().join(current_dir()?).join(TOML_NAME);

    Ok(toml)
}

/// Returns the current working directory.
fn current_dir() -> Result<PathBuf> {
    Ok(env::current_dir().context("Could not get current directory")?)
}
