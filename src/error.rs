use crate::cli::CliErrorKind;
use crate::frontend::FrontEndError;
use crate::sources::SourcesError;


#[derive(thiserror::Error, Debug)]
pub enum PloyErrorKind {

    #[error(transparent)]
    Cli(#[from] CliErrorKind),

    #[error(transparent)]
    FrontEnd(FrontEndError),

    #[error(transparent)] 
    SourceError(SourcesError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
