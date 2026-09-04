//! Command-line argument parsing.
//!
//! Defines the CLI structure for launching Brainstorm in development mode
//! with optional Tuna cloud secrets integration.

use clap::{Parser, Subcommand};

/// Launch App in development mode with Tuna cloud secrets.
#[derive(Debug)]
pub(crate) struct DjUp {
    pub(crate) command: Command,
}

/// Internal CLI structure used for parsing command-line arguments.
#[derive(Debug, Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
    /// subcommand to execute (defaults to `runserver`)
    #[command(subcommand)]
    command: Option<Command>,
}

/// Available subcommands for the Dj utility.
#[derive(Debug, Subcommand, Default)]
pub(crate) enum Command {
    /// Start the Django development server.
    #[default]
    Runserver,

    /// Run arbitrary `manage.py` commands.
    Manage {
        /// django command and its flags (everything that follows `manage`)
        #[arg(allow_hyphen_values = true, trailing_var_arg = true)]
        django_args: Vec<String>,
    },
}

/// Parse command-line arguments and return the configuration.
///
/// If no subcommand is provided, `runserver` is used as the default.
pub(crate) fn parse_args() -> DjUp {
    let cli = Cli::parse();
    DjUp {
        command: cli.command.unwrap_or_default(),
    }
}
