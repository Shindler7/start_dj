//! Command-line argument parsing.
//!
//! Defines the CLI structure for launching Brainstorm in development mode
//! with optional Tuna cloud secrets integration.

use argh::FromArgs;

/// Launch App in development mode with Tuna cloud secrets.
#[derive(Debug, FromArgs)]
pub(crate) struct DjUp {
    /// subcommand to execute (defaults to `runserver`).
    #[argh(subcommand)]
    pub(crate) command: Option<Command>,
}

/// Available subcommands for the Dj utility.
#[derive(Debug, FromArgs)]
#[argh(subcommand)]
pub(crate) enum Command {
    Runserver(Runserver),
    Manage(Manage),
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "runserver")]
/// Start the Django development server.
pub(crate) struct Runserver {}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "manage")]
/// Run arbitrary `manage.py` commands.
pub(crate) struct Manage {
    /// django command and its flags (everything that follows `manage`).
    #[argh(positional, greedy)]
    pub(crate) django_args: Vec<String>,
}

impl Default for Command {
    fn default() -> Self {
        Self::Runserver(Runserver {})
    }
}

/// Parse command-line arguments and return the configuration.
///
/// If no subcommand is provided, `runserver` is used as the default.
pub(crate) fn parse_args() -> DjUp {
    let mut args: DjUp = argh::from_env();
    args.command.get_or_insert_with(Command::default);
    args
}
