use crate::cli::CliErrorKind;
use crate::frontend::FrontEndError;
use crate::sources::{SourcesError, FileSpan};

#[derive(thiserror::Error)]

pub enum PloyErrorKind {
    #[error(transparent)]
    Cli(#[from] CliErrorKind),

    #[error("{0}")]
    FrontEnd(#[from] FrontEndError),

    #[error(transparent)] 
    SourceError(#[from] SourcesError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl std::fmt::Debug for PloyErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(e) => write!(f,"{}", e.to_string()),
            _ => write!(f,"{}",self)
        }
    }
}


