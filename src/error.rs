#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("command cannot be empty")]
    EmptyCommand,
    #[error("cannot extract a name from command `{0}`")]
    InvalidCommand(String),
    #[error("too many items with the same name `{0}`")]
    TooManyItems(String),
    #[error("invalid id `{0}`: must not be empty or contain path separators")]
    InvalidId(String),
    #[error("command already exists with id `{0}` (use --force to add it anyway)")]
    AlreadyExists(String),
    #[error("config file id `{0}` not found")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;
