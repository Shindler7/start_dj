//! Command builders — construct command-line arguments for various tools.

use crate::parse_toml::models::TunaParams;

/// Builds the command-line arguments for the `tuna secrets run` command.
///
/// Returns a vector of strings readies to be passed to `std::process::Command`.
pub(super) fn tuna_args(tuna_params: &TunaParams) -> Vec<String> {
    [
        "tuna",
        "secrets",
        "run",
        "--project",
        &tuna_params.project,
        "--config",
        &tuna_params.config,
        "--api-key",
        tuna_params.api_key(),
        "--",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Builds the command-line arguments for `uv run`.
pub(super) fn uv_args() -> Vec<String> {
    ["uv", "run"].into_iter().map(String::from).collect()
}
