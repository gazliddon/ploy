use crate::cli::CliErrorKind;

#[derive(thiserror::Error, Debug)]
pub enum FileErrorKind {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("File already exists: {0}")]
    AlreadyExists(String),
}

#[derive(thiserror::Error, Debug)]
pub enum PloyErrorKind {
    #[error(transparent)]
    FileError(#[from] FileErrorKind),
    #[error(transparent)]
    Cli(#[from] CliErrorKind),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
