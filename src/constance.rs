//! Константы приложения.
use std::path::PathBuf;

// Имя конфигруационного файла с параметрами запуска.
const TOML_NAME: &str = "start.toml";

/// Полный путь к файлу конфигурации.
pub(crate) fn toml_path() -> PathBuf {
    PathBuf::new()
        .join(env!("CARGO_MANIFEST_DIR"))
        .join(TOML_NAME)
}
