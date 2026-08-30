//! TOML configuration schema for the launcher.
//!
//! This module defines strongly typed config sections deserialized with Serde.
//! Sections and fields with defaults are optional in TOML; omitted values fall
//! back to sensible defaults. Sensitive values are redacted in `Debug` output.

use crate::parse_toml::types::DjangoCommands;
use serde::Deserialize;
use std::{fmt::Debug, path::PathBuf};

/// Specifies where environment variables should be loaded from.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub(super) enum EnvSource {
    /// Load from a `.env` file only.
    #[serde(alias = "Dotenv", alias = "DOTENV")]
    Dotenv,

    /// Load from the system environment only.
    #[serde(alias = "Env", alias = "ENV")]
    Env,

    /// Load from both — `.env` file and system environment.
    #[serde(alias = "Mixed", alias = "MIXED")]
    Mixed,
}

impl Default for EnvSource {
    fn default() -> Self {
        Self::Dotenv
    }
}

/// Specify the runtime environment (optional).
#[derive(Deserialize, Debug)]
pub(super) struct Environment {
    /// Environment file path (e.g., `.env`, `settings.env`).
    pub(super) env: PathBuf,

    /// Source of variables for ${VAR} interpolation.
    #[serde(default)]
    pub(super) source: EnvSource,
}

/// Root configuration structure for the application.
/// Holds optional environment settings.
#[derive(Deserialize, Debug)]
pub(super) struct BootstrapConfig {
    /// Specify the runtime environment (optional)
    pub(super) environment: Option<Environment>,
}

/// Command-line parameters for running a Django application.
#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub(crate) struct Params {
    /// Base settings for launching `Django`.
    pub(crate) django: Django,

    /// Credentials for accessing the `Tuna Secrets` cloud (completely optional).
    pub(crate) tuna: Option<TunaParams>,

    /// Feature flags that control optional functionality.
    pub(crate) features: Features,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct Django {
    /// Core Django launch configuration.
    #[serde(rename = "runserver")]
    run_server: DjangoCommands,

    /// Port to bind the development server to.
    pub(crate) port: u16,

    /// Use IPv6 address for the server.
    pub(crate) ipv6: bool,

    /// Disable multi-threading.
    #[serde(rename = "nothreading")]
    pub(crate) no_threading: bool,

    /// Disable auto-reload on file changes.
    #[serde(rename = "noreload")]
    pub(crate) no_reload: bool,

    /// Disable serving static files via Django.
    #[serde(rename = "nostatic")]
    pub(crate) no_static: bool,

    /// Run server in insecure mode (allows serving over HTTP in production).
    pub(crate) insecure: bool,

    /// Skip system checks before running the server.
    #[serde(rename = "skip-checks")]
    pub(crate) skip_checks: bool,
}

impl Default for Django {
    fn default() -> Self {
        Self {
            run_server: DjangoCommands::default(),
            port: 8000,
            ipv6: false,
            no_threading: false,
            no_reload: false,
            no_static: false,
            insecure: false,
            skip_checks: false,
        }
    }
}

impl Django {
    /// Builds the list of command-line arguments for the Django `runserver` command.
    ///
    /// Iterates through all boolean flags and appends the corresponding
    /// command-line option if the flag is enabled.
    pub(crate) fn runserver_args(&self) -> Vec<String> {
        let flags = [
            (self.ipv6, "--ipv6"),
            (self.no_threading, "--nothreading"),
            (self.no_reload, "--noreload"),
            (self.no_static, "--nostatic"),
            (self.insecure, "--insecure"),
            (self.skip_checks, "--skip-checks"),
        ];

        flags
            .iter()
            .filter_map(|(condition, flag)| condition.then(|| flag.to_string()))
            .collect()
    }

    /// Returns clone of runserver-field.
    pub(crate) fn runserver(&self) -> DjangoCommands {
        self.run_server.clone()
    }
}

/// Credentials and configuration for accessing the Tuna cloud service.
#[derive(Deserialize)]
pub(crate) struct TunaParams {
    // Tuna project name.
    pub(crate) project: String,

    /// Tuna configuration profile (e.g., "dev", "staging", "prod").
    pub(crate) config: String,

    /// API key for authenticating with Tuna — kept private.
    api_key: String,
}

impl Debug for TunaParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TunaParams")
            .field("project", &self.project)
            .field("config", &self.config)
            .field("api_key", &"<redacted>")
            .finish()
    }
}

impl TunaParams {
    /// Returns the API key for use in authenticated requests.
    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }
}

/// Feature flags for enabling/disabling optional capabilities.
#[derive(Deserialize, Debug)]
#[serde(default)]
pub(crate) struct Features {
    /// Enable Tuna cloud secrets integration.
    pub(crate) tuna: bool,

    /// Enable `uv` package manager support.
    pub(crate) uv: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            tuna: false,
            uv: false,
        }
    }
}
