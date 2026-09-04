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
mod commands;
mod constants;
mod executor;
mod parse_toml;

use anyhow::Result as AnyhowResult;
use cli::{Command as ArgsCommand, parse_args};
use env_logger::WriteStyle;
use log::error;
use parse_toml::read_params;
use std::{io::Write, process::ExitCode};

/// Application entry point.
///
/// Initializes logging, runs the main application logic, and returns
/// an appropriate exit code.
fn main() -> ExitCode {
    env_logger::builder()
        .format_timestamp(None)
        .format(|buf, record| writeln!(buf, "[{}]: {}", record.level(), record.args()))
        .write_style(WriteStyle::Auto)
        .init();

    dj_start().unwrap_or_else(|err| {
        error!("{}", err);
        ExitCode::FAILURE
    })
}

/// Core application logic.
///
/// Loads configuration, parses command-line arguments, and executes
/// the requested command (`runserver` or `manage`).
fn dj_start() -> AnyhowResult<ExitCode> {
    let params = read_params()?;
    let command = parse_args().command;

    log::debug!(
        "TOML file loaded successfully, now executing command `{:?}`",
        command
    );

    match command {
        ArgsCommand::Manage { django_args } => {
            let django_commands = django_args.into();
            executor::manage(&params, &django_commands)
        }
        ArgsCommand::Runserver => executor::run_server(&params),
    }
}
