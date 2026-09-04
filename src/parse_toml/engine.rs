//! Parser for configuration parameters from `start.toml`.

use crate::{
    constants,
    parse_toml::models::{BootstrapConfig, EnvSource, Params},
};
use anyhow::{Context, Result as AnyhowResult, bail};
use regex::Regex;
use std::{collections::HashMap, fs::read_to_string, path::Path, sync::LazyLock};
use toml::Value;

/// Regex for detecting and extracting `${VAR}`-style placeholders.
///
/// Matches patterns like `${API_KEY}`, `${DATABASE_URL}`, etc.
/// The capturing group extracts the variable name for lookup.
static RE_TOML: LazyLock<Regex> = {
    LazyLock::new(|| {
        Regex::new(r"\$\{([A-Za-z_][A-Za-z0-9_]*)}").expect("Invalid placeholder regex")
    })
};

/// Loads environment variables from a `.env` file into a hash map.
///
/// ## Errors
/// Fails if the file doesn't exist, can't be read, or contains invalid entries.
fn load_dotenv_map(path: &Path) -> AnyhowResult<HashMap<String, String>> {
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
fn collect_vars(source: &EnvSource, env_path: &Path) -> AnyhowResult<HashMap<String, String>> {
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
fn expand_placeholders(input: &str, vars: &HashMap<String, String>) -> AnyhowResult<String> {
    let re = &*RE_TOML;

    let mut missing_key = None;

    let result = re.replace_all(input, |caps: &regex::Captures| {
        let key = &caps[1];

        match vars.get(key) {
            Some(val) => val.clone(),
            None => {
                missing_key = Some(key.to_string());
                String::new()
            }
        }
    });

    if let Some(key) = missing_key {
        bail!("Variable `{}` is referenced in TOML but not found", key);
    }

    Ok(result.into_owned())
}

fn has_placeholders(value: &Value) -> bool {
    match value {
        Value::String(s) => s.contains("${"),
        Value::Array(items) => items.iter().any(has_placeholders),
        Value::Table(map) => map.values().any(has_placeholders),
        _ => false,
    }
}

/// Recursively walks a TOML value and expands `${VAR}` placeholders in all strings.
///
/// Supports nested structures: strings inside arrays and tables are also processed.
fn interpolate_value(v: &mut Value, vars: &HashMap<String, String>) -> AnyhowResult<()> {
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

/// Reads and parses the configuration file into `Params`.
///
/// ## Panics
/// Exits the process with an error message if the TOML file doesn't exist,
/// cannot be read, or fails to parse.
pub(crate) fn read_params() -> AnyhowResult<Params> {
    let toml_path = constants::toml_path()?;

    // Check that the config file actually exists.
    if !toml_path.is_file() {
        bail!("The TOML file `{}` does not exist.", toml_path.display());
    }

    let toml_string = read_to_string(toml_path).context("Failed to read TOML file")?;

    let bootstrap: BootstrapConfig = toml::from_str(&toml_string)
        .context("Failed to parse TOML (bootstrap pass for [environment])")?;

    let mut doc: Value = toml::from_str(&toml_string).context("Failed to parse TOML")?;

    // 1) Read Env and substitute `${VAR}` placeholders from `.env` — only if needed.
    if has_placeholders(&doc) {
        let env_cfg = bootstrap
            .environment
            .as_ref()
            .context("Found `${VAR}` placeholders, but `[environment]` section is missing")?;

        let vars = collect_vars(&env_cfg.source, &env_cfg.env)?;
        interpolate_value(&mut doc, &vars)?;
    }

    // 2) Finalize.
    let params: Params = doc.try_into().context("Failed to deserialize TOML file")?;

    Ok(params)
}
