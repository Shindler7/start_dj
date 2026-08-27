//! Разбор параметров командной строки.

use argh::FromArgs;

/// Запуск Brainstorm в режиме разработки с облачными секретами Tuna.
#[derive(Debug, FromArgs)]
pub(crate) struct DjUp {
    /// отключить использование `TUNA` при запуске.
    #[argh(switch)]
    pub(crate) no_tuna: bool,

    /// режим запуска.
    #[argh(subcommand)]
    pub(crate) command: Option<Command>,
}

/// Подкоманды запуска.
#[derive(Debug, FromArgs)]
#[argh(subcommand)]
pub(crate) enum Command {
    Runserver(Runserver),
    Manage(Manage),
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "runserver")]
/// Запуск ASGI сервера (uvicorn).
pub(crate) struct Runserver {}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "manage")]
/// Запуск команд manage.py.
pub(crate) struct Manage {
    /// команда Django и её флаги.
    #[argh(positional, greedy)]
    pub(crate) django_args: Vec<String>,
}

impl Default for Command {
    fn default() -> Self {
        Self::Runserver(Runserver {})
    }
}

pub(crate) fn parse_args() -> DjUp {
    let mut args: DjUp = argh::from_env();
    args.command.get_or_insert_with(Command::default); // runserver по умолчанию
    args
}
