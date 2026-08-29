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
mod dj_toml;
mod types;

use crate::{
    dj_toml::{Params, read_params},
    types::DjangoCommands,
};
use anyhow::{Result, bail};
use cli::{Command as ArgsCommand, DjUp, parse_args};
use std::process::{Command, ExitCode};

/// Program entry point — parses CLI args, loads config, and dispatches
/// to the appropriate handler.
fn main() -> Result<ExitCode> {
    let dj_up: DjUp = parse_args();
    let params = read_params()?;

    let result = match dj_up.command {
        Some(ArgsCommand::Manage(m)) => {
            let django_commands: DjangoCommands = m.django_args.into();
            manage(&params, &django_commands)
        }
        _ => run_server(&params),
    };

    Ok(result?)
}

/// Prepends feature-related commands (Tuna, uv) to the Django command list.
///
/// If Tuna is enabled, it injects `tuna secrets run ...` with credentials.
/// If uv is enabled, it injects `uv run ...`.
///
/// Feature commands are always prepended before the actual Django command.
fn update_commands_by_features(
    django_commands: DjangoCommands,
    params: &Params,
) -> Result<DjangoCommands> {
    let mut features = DjangoCommands::new();

    if params.features.tuna {
        match &params.tuna {
            Some(tuna_params) => features.extend(vec![
                "tuna".to_string(),
                "secrets".to_string(),
                "run".to_string(),
                "--project".to_string(),
                tuna_params.project.to_string(),
                "--config".to_string(),
                tuna_params.config.to_string(),
                "--api-key".to_string(),
                tuna_params.api_key().to_string(),
                "--".to_string(),
            ]),
            None => bail!(
                "Tuna feature is enabled (`features.tuna = true`), \
                but the `[tuna]` configuration block is missing. \
                Either add a `[tuna]` section to your `start.toml` or set `features.tuna = false`."
            ),
        }
    }

    if params.features.uv {
        features.extend(["uv".to_string(), "run".to_string()]);
    }

    if features.is_empty() {
        return Ok(django_commands);
    }

    features.extend(django_commands.iter().cloned());

    Ok(features)
}

/// Starts the Django development server with the provided configuration.
fn run_server(params: &Params) -> Result<ExitCode> {
    let mut django_commands = params.django.runserver();

    if django_commands.is_default() {
        django_commands.extend(params.django.runserver_args().into_iter());
        django_commands.push(params.django.port())
    };

    django_commands = update_commands_by_features(django_commands, &params)?;

    command_execute(Command::from(django_commands))
}

/// Executes a `manage.py` command with the given arguments.
fn manage(params: &Params, django_args: &DjangoCommands) -> Result<ExitCode> {
    let mut django_commands = DjangoCommands::new();

    if !params.features.uv {
        django_commands.push("python".to_string());
    }

    django_commands.push("manage.py".to_string());
    django_commands.extend(django_args.iter().cloned());

    django_commands = update_commands_by_features(django_commands, &params)?;

    command_execute(Command::from(django_commands))
}

/// Spawns a child process, waits for it to complete, and returns the exit status.
///
/// If the process fails to spawn or errors during execution, it's killed and
/// a failure code is returned.
fn command_execute(mut command: Command) -> Result<ExitCode> {
    match command.status() {
        Ok(status) => {
            if status.success() {
                Ok(ExitCode::SUCCESS)
            } else {
                eprintln!(
                    "Command exited with non-zero status: {}",
                    status
                        .code()
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "terminated by signal".to_string())
                );
                Ok(ExitCode::FAILURE)
            }
        }
        Err(err) => {
            eprintln!("Failed to start command: {err}");
            Ok(ExitCode::FAILURE)
        }
    }
}
