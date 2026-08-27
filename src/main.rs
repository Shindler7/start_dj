//! Консольное приложение для запуска Django-приложения.
pub(crate) mod constance;
mod dj_toml;

use crate::dj_toml::read_params;
use anyhow::Result;

fn main() -> Result<()> {
    println!("Hello, world!");

    let params = read_params()?;
    
    println!("{:#?}", params);

    Ok(())
}
