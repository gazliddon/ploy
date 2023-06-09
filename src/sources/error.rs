use super::SearchPathsError;

#[derive(Clone, thiserror::Error, Debug)]
pub enum SourcesError {
    #[error(transparent)]
    SearchPathsError(#[from] SearchPathsError),

    #[error("Source already in database")]
    SourceIsAlreadyInDatabase,
}
