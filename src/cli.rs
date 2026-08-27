//! Разборка параметров командной строки.

use argh::FromArgs;

#[derive(Debug, FromArgs)]
/// Запуск приложения в режиме разработки.
pub(crate) struct DjUp {
    /// отключить использование `TUNA` при запуске.
    #[argh(option, default = "false")]
    pub(crate) no_tuna: bool,
}
