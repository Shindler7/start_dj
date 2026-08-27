//! Парсер параметров из файла `start.toml`.

use crate::constance;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::exit;

/// Параметры запуска приложения Django.
#[derive(Deserialize, Debug)]
pub(crate) struct Params {
    /// Серверные настройки.
    #[serde(default)]
    server: ServerParams,
    /// Данные для доступа к облаку Tuna Secrets.
    #[serde(default)]
    tuna: TunaParams,
}

/// Параметры для конфигурации запуска сервера.
#[derive(Debug, Deserialize)]
pub(crate) struct ServerParams {
    ///
    #[serde(default = "default_port")]
    port: u16,
    #[serde(default)]
    reload: bool,
}

impl Default for ServerParams {
    fn default() -> Self {
        Self {
            port: default_port(),
            reload: false,
        }
    }
}

/// Значение порта по умолчанию: `8080`.
fn default_port() -> u16 {
    8000
}

/// Параметры доступа к облаку Tuna.
#[derive(Debug, Deserialize)]
#[serde(default)]
pub(crate) struct TunaParams {
    project: String,
    config: String,
    api_key: String,
}

impl Default for TunaParams {
    fn default() -> Self {
        Self {
            project: "".to_string(),
            config: "".to_string(),
            api_key: "".to_string(),
        }
    }
}

/// Чтение параметров из конфигурационного файла.
pub(crate) fn read_params() -> Result<Params> {
    let toml_path = constance::toml_path();

    // Проверка существования файла конфигурации.
    if !toml_path.exists() {
        eprintln!("The TOML file `{}` does not exist", toml_path.display());
        exit(1);
    }

    let toml_string = std::fs::read_to_string(toml_path).context("Failed to read TOML file")?;

    let params: Params = toml::from_str(&toml_string).context("Failed to parse TOML file")?;

    Ok(params)
}
