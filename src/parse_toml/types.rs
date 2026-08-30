//! Core data structures used throughout the application.

use anyhow::{Context, Result as AnyhowResult};
use serde::Deserialize;
use std::{
    ops::{Deref, DerefMut},
    process::Command,
};

/// Default Django management command for starting the development server.
const DEFAULT_RUN_DJANGO: &str = "runserver";

/// Container for additional command-line arguments to pass through.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub(crate) struct DjangoCommands(Vec<String>);

impl From<Vec<String>> for DjangoCommands {
    fn from(vec: Vec<String>) -> Self {
        Self(vec)
    }
}

impl From<Vec<&str>> for DjangoCommands {
    fn from(vec: Vec<&str>) -> Self {
        Self(vec.iter().map(|s| s.to_string()).collect())
    }
}

impl TryFrom<DjangoCommands> for Command {
    type Error = anyhow::Error;

    fn try_from(dj_commands: DjangoCommands) -> AnyhowResult<Self> {
        let mut parts = dj_commands.0.into_iter();
        let program = parts.next().context("Django command is empty")?;

        let mut command = Command::new(&program);
        command.args(parts);
        Ok(command)
    }
}

impl Default for DjangoCommands {
    fn default() -> Self {
        Self(vec![DEFAULT_RUN_DJANGO.to_string()])
    }
}

impl Deref for DjangoCommands {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DjangoCommands {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DjangoCommands {
    // Creates an empty list of Django commands.
    pub(crate) fn new() -> Self {
        Self(Vec::new())
    }

    /// Check if the command is the default one.
    pub(crate) fn is_default(&self) -> bool {
        self.0.len() == 1 && self.0[0] == DEFAULT_RUN_DJANGO
    }
}
