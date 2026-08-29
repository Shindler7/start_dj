//! Parser for configuration parameters from `start.toml`.

use crate::{constance, types::DjangoCommands};
use anyhow::{Context, Result, bail};
use regex::Regex;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fmt::Debug,
    path::{Path, PathBuf},
};
use toml::Value;

/// Specifies where environment variables should be loaded from.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
enum EnvSource {
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
pub(crate) struct Environment {
    /// Environment file path (e.g., `.env`, `settings.env`).
    #[serde(default = "default_env_path")]
    env: PathBuf,

    /// Source of variables for ${VAR} interpolation.
    #[serde(default)]
    source: EnvSource,
}

/// Returns the default path to the environment file.
fn default_env_path() -> PathBuf {
    PathBuf::from(".env")
}

/// Root configuration structure for the application.
/// Holds optional environment settings.
#[derive(Deserialize, Debug)]
struct BootstrapConfig {
    /// Specify the runtime environment (optional)
    environment: Option<Environment>,
}

/// Loads environment variables from a `.env` file into a hash map.
///
/// ## Errors
/// Fails if the file doesn't exist, can't be read, or contains invalid entries.
fn load_dotenv_map(path: &Path) -> Result<HashMap<String, String>> {
    if !path.exists() {
        bail!("Environment file `{}` does not exist", path.display());
    }

    let mut vars = HashMap::new();
    for item in dotenvy::from_path_iter(path)
        .with_context(|| format!("Failed to read env file `{}`", path.display()))?
    {
        let (k, v) =
            item.with_context(|| format!("Failed to parse env entry in `{}`", path.display()))?;
        vars.insert(k, v);
    }
    Ok(vars)
}

/// Collects environment variables from the specified source(s).
///
/// For `Mixed` mode, the `.env` file is optional — if it doesn't exist, we just start
/// with an empty map and fill it with system variables. System environment variables
/// always take precedence over `.env` in `Mixed` mode.
fn collect_vars(source: EnvSource, env_path: &Path) -> Result<HashMap<String, String>> {
    let mut vars = match source {
        EnvSource::Dotenv => load_dotenv_map(env_path)?,
        EnvSource::Env => HashMap::new(),
        EnvSource::Mixed => {
            // Mixed: .env is optional — if missing, start with an empty base.
            if env_path.exists() {
                load_dotenv_map(env_path)?
            } else {
                HashMap::new()
            }
        }
    };

    // For `Env` and `Mixed`, merge in system environment variables.
    // In `Mixed` mode, system vars override `.env` values.
    if matches!(source, EnvSource::Env | EnvSource::Mixed) {
        for (k, v) in std::env::vars() {
            vars.insert(k, v);
        }
    }

    Ok(vars)
}

/// Expands `${VAR}` placeholders in a string using values from the environment map.
///
/// ## Errors
/// Fails if a referenced variable is not found in the map.
fn expand_placeholders(input: &str, vars: &HashMap<String, String>) -> Result<String> {
    let re = Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)}").context("Invalid placeholder regex")?;

    let mut out = String::with_capacity(input.len());
    let mut last = 0;

    for caps in re.captures_iter(input) {
        let m = caps.get(0).unwrap();
        let key = caps.get(1).unwrap().as_str();

        // Everything between the last match and this one goes in verbatim.
        out.push_str(&input[last..m.start()]);

        // Look up the variable and substitute, or fail hard.
        if let Some(value) = vars.get(key) {
            out.push_str(value);
        } else {
            bail!("Variable `{}` is referenced in TOML but not found", key);
        }
        last = m.end();
    }

    out.push_str(&input[last..]);
    Ok(out)
}

/// Command-line parameters for running a Django application.
#[derive(Deserialize, Debug)]
pub(crate) struct Params {
    /// Base settings for launching `Django`.
    pub(crate) django: Django,

    /// Credentials for accessing the `Tuna Secrets` cloud (completely optional).
    pub(crate) tuna: Option<TunaParams>,

    /// Feature flags that control optional functionality.
    pub(crate) features: Features,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Django {
    /// Core Django launch configuration.
    #[serde(rename = "runserver", default)]
    run_server: DjangoCommands,

    /// Port to bind the development server to.
    #[serde(default)]
    pub(crate) port: u16,

    /// Use IPv6 address for the server.
    #[serde(default)]
    pub(crate) ipv6: bool,

    /// Disable multi-threading.
    #[serde(rename = "nothreading", default)]
    pub(crate) no_threading: bool,

    /// Disable auto-reload on file changes.
    #[serde(rename = "noreload", default)]
    pub(crate) no_reload: bool,

    /// Disable serving static files via Django.
    #[serde(rename = "nostatic", default)]
    pub(crate) no_static: bool,

    /// Run server in insecure mode (allows serving over HTTP in production).
    #[serde(default)]
    pub(crate) insecure: bool,

    /// Skip system checks before running the server.
    #[serde(rename = "skip-checks", default)]
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

    /// Returns the port as a string.
    pub(crate) fn port(&self) -> String {
        self.port.to_string()
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

/// Reads and parses the configuration file into `Params`.
///
/// ## Panics
/// Exits the process with an error message if the TOML file doesn't exist,
/// cannot be read, or fails to parse.
pub(crate) fn read_params() -> Result<Params> {
    let toml_path = constance::toml_path()?;

    // Check that the config file actually exists.
    if !toml_path.exists() {
        bail!("The TOML file `{}` does not exist", toml_path.display());
    }

    let toml_string = std::fs::read_to_string(toml_path).context("Failed to read TOML file")?;

    let bootstrap: BootstrapConfig = toml::from_str(&toml_string)
        .context("Failed to parse TOML (bootstrap pass for [environment])")?;

    // 1) Read env.
    let env_cfg = bootstrap.environment.unwrap_or(Environment {
        env: default_env_path(),
        source: EnvSource::Dotenv,
    });

    let mut doc: Value = toml::from_str(&toml_string).context("Failed to parse TOML")?;

    // 2) Substitute `${VAR}` placeholders from `.env` — only if needed.
    if toml_string.contains("${") {
        let vars = collect_vars(env_cfg.source, &env_cfg.env)?;
        interpolate_value(&mut doc, &vars)?;
    }

    // 3) Finalize.
    let params: Params = doc.try_into().context("Failed to deserialize TOML file")?;

    Ok(params)
}

/// Recursively walks a TOML value and expands `${VAR}` placeholders in all strings.
///
/// Supports nested structures: strings inside arrays and tables are also processed.
fn interpolate_value(v: &mut Value, vars: &HashMap<String, String>) -> Result<()> {
    match v {
        Value::String(s) => {
            *s = expand_placeholders(s, vars)?;
        }
        Value::Array(arr) => {
            for item in arr {
                interpolate_value(item, vars)?;
            }
        }
        Value::Table(tbl) => {
            for (_, val) in tbl.iter_mut() {
                interpolate_value(val, vars)?;
            }
        }
        _ => {}
    }
    Ok(())
}
