use thin_vec::ThinVec;
use thiserror::Error;
use std::path::PathBuf;

use super::SearchPathsError;

#[derive(Clone, Error, Debug)]
pub enum SourcesError {
    #[error(transparent)]
    SearchPathsError(#[from] SearchPathsError),

    #[error("Source already in database")]
    SourceIsAlreadyInDatabase,
}
