use thiserror::Error;

#[derive(Debug,Error)]
pub enum FrontEndError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
