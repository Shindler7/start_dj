//! Command-line argument parsing.
//!
//! Defines the CLI structure for launching Brainstorm in development mode
//! with optional Tuna cloud secrets integration.

use clap::{Parser, Subcommand, error::ErrorKind};

/// Launch App in development mode with Tuna cloud secrets.
#[derive(Debug)]
pub(crate) struct DjUp {
    pub(crate) command: Command,
}

/// Internal CLI structure used for parsing command-line arguments.
#[derive(Debug, Parser)]
#[command(
    version,
    about,
    long_about = None,
    override_usage = "dj [OPTIONS] [MANAGE_ARGS]...",
    after_help = format!(r#"{bold}{underline}Django Manage Proxy Modes{reset}:

Any unknown command is automatically proxied to `manage.py`.

Examples:
  dj <command>    Proxy any arbitrary command to python manage.py <command>
  dj m            Alias for `migrate`
  dj mm           Alias for `makemigrations`
  dj s            Alias for `shell`"#,
    bold="\x1b[1m",
    underline="\x1b[4m",
    reset="\x1b[0m")
)]
struct Cli {
    /// subcommand to execute (defaults to `runserver`)
    #[command(subcommand)]
    command: Option<Command>,
}

/// Parse CLI args into a `DjUp` struct.
///
/// - If the user provides a valid subcommand, use it.
/// - If no subcommand is given, default to `runserver`.
/// - If the subcommand is unknown, treat it as `manage` and pass the raw args.
#[derive(Debug, Subcommand, Default)]
pub(crate) enum Command {
    /// Start the Django development server (default).
    #[default]
    #[command(visible_alias = "run")]
    Runserver,

    /// Run any `manage.py` command — just pass it along.
    #[command(external_subcommand)]
    Manage(Vec<String>),
}

/// Parse command-line arguments and return the configuration.
///
/// If no subcommand is provided, `runserver` is used as the default.
pub(crate) fn parse_args() -> DjUp {
    let mut command = match Cli::try_parse() {
        Ok(cli) => cli.command.unwrap_or_default(),
        Err(err) => {
            if err.kind() == ErrorKind::DisplayHelp
                || err.kind() == ErrorKind::DisplayVersion
                || err.kind() == ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            {
                err.exit()
            }

            // Fallback: anything else becomes `manage` with the raw args.
            let raw_args: Vec<String> = std::env::args().skip(1).collect();
            Command::Manage(raw_args)
        }
    };

    if let Command::Manage(ref mut args) = command
        && let Some(first_arg) = args.first()
    {
        let expanded = match first_arg.as_str() {
            "m" => Some("migrate"),
            "mm" => Some("makemigrations"),
            "s" => Some("shell"),
            _ => None,
        };

        if let Some(replacement) = expanded {
            args[0] = replacement.to_string();
        }
    }

    DjUp { command }
}
