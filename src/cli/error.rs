#[derive(thiserror::Error, Debug)]
pub enum CliErrorKind {
    #[error("Unrecognised command {0}")]
    UnrecognisedCommand(String),
    #[error("Need a build action")]
    NoAction,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
