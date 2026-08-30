//! Command execution logic — runs Django server and management commands.
//!
//! This module handles building the final command line, prepending feature
//! wrappers (Tuna, uv), and executing the resulting process.

use crate::{
    constance::{MANAGE_PY, PYTHON_BIN},
    parse_toml::{DjangoCommands, Params},
};
use anyhow::{Result as AnyhowResult, bail};
use std::process::{Command, ExitCode, ExitStatus, Stdio};

/// Prepends feature-related commands (Tuna, uv) to the Django command list.
///
/// If Tuna is enabled, it injects `tuna secrets run ...` with credentials.
/// If uv is enabled, it injects `uv run ...`.
///
/// Feature commands are always prepended before the actual Django command.
fn update_commands_by_features(
    django_commands: DjangoCommands,
    params: &Params,
) -> AnyhowResult<DjangoCommands> {
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
pub(super) fn run_server(params: &Params) -> AnyhowResult<ExitCode> {
    let mut django_commands = params.django.runserver();

    if django_commands.is_default() {
        django_commands.push(MANAGE_PY.to_string());
        django_commands.extend(params.django.runserver_args().into_iter());
        django_commands.push(params.django.port.to_string());
    };

    django_commands = update_commands_by_features(django_commands, &params)?;

    command_execute(Command::try_from(django_commands)?)
}

/// Executes a `manage.py` command with the given arguments.
pub(super) fn manage(params: &Params, django_args: &DjangoCommands) -> AnyhowResult<ExitCode> {
    let mut django_commands = DjangoCommands::new();

    if !params.features.uv {
        django_commands.push(PYTHON_BIN.to_string());
    }

    django_commands.push(MANAGE_PY.to_string());
    django_commands.extend(django_args.iter().cloned());

    django_commands = update_commands_by_features(django_commands, &params)?;

    command_execute(Command::try_from(django_commands)?)
}

/// Spawns a child process, waits for it to complete, and returns the exit status.
///
/// If the process fails to spawn or errors during execution, it's killed and
/// a failure code is returned.
fn command_execute(mut command: Command) -> AnyhowResult<ExitCode> {
    fn format_status(status: ExitStatus) -> String {
        status
            .code()
            .map(|code| format!("exit code {code}"))
            .unwrap_or_else(|| "terminated".to_string())
    }

    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    match command.status() {
        Ok(status) if status.success() => Ok(ExitCode::SUCCESS),
        Ok(status) => {
            eprintln!("Command failed: {}", format_status(status));
            Ok(ExitCode::FAILURE)
        }
        Err(err) => {
            eprintln!(
                "Failed to run `{}`: {err}",
                command.get_program().to_string_lossy()
            );
            Ok(ExitCode::FAILURE)
        }
    }
}
