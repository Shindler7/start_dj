//! Console utility for running Django applications.
//!
//! This tool expects a `start.toml` configuration file to be present.
//! Here's a sample configuration with all available options:
//!
//! ```toml
//! [environment]
//! env = ".env"
//!
//! [django]
//! runserver = ["python", "-m", "uvicorn", "--reload", "rustogrped.asgi:application"]
//! port = 8080
//! ipv6 = false
//! nothreading = false
//! noreload = false
//! nostatic = false
//! insecure = false
//! skip_checks = false
//!
//! [tuna]
//! project = "brainstorm"
//! config = "dev"
//! api_key = "${TUNA_API_KEY}"
//! ```
//!
//! All sections are optional — if you omit any (or all) of them,
//! the tool falls back to sensible defaults. The example above
//! shows the default values for every field.
mod cli;
mod constance;
mod executor;
mod parse_toml;
mod commands;

use anyhow::Result as AnyhowResult;
use cli::{Command as ArgsCommand, DjUp, parse_args};
use parse_toml::{DjangoCommands, read_params};
use std::process::ExitCode;

/// Program entry point — parses CLI args, loads config, and dispatches
/// to the appropriate handler.
fn main() -> AnyhowResult<ExitCode> {
    let dj_up: DjUp = parse_args();
    let params = read_params()?;

    let result = match dj_up.command {
        Some(ArgsCommand::Manage(m)) => {
            let django_commands: DjangoCommands = m.django_args.into();
            executor::manage(&params, &django_commands)
        }
        _ => executor::run_server(&params),
    };

    Ok(result?)
}
