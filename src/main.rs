//! Консольное приложение для запуска Django-приложения.
//!
//! Обязательно наличие файла `start.toml`. Примерная конфигурация:
//!
//! ```text
//! [server]
// port = 8080
// reload = true
//
// [tuna]
// project = ""
// config = ""
// api_key = ""
//! ```
//!
//! Если отдельные блоки (или все вместе) отсутствуют, то устанавливаются
//! значения по умолчанию. Все дефолтные параметры указаны в примере.
mod cli;
mod constance;
mod dj_toml;

use crate::dj_toml::read_params;
use anyhow::Result;
use cli::DjUp;

fn main() -> Result<()> {
    let params = read_params()?;

    let dj_up: DjUp = argh::from_env();

    println!("{:#?}", params);

    Ok(())
}
