use super::syntax::SyntaxErrorKind;
use thiserror::Error;
use unraveler::{ParseError, ParseErrorKind, Severity};
use crate::sources::SearchPathsError;
use super::prelude::*;


pub type PResult<'a, O, E = FrontEndError> = Result<(Span<'a>, O), E>;

#[derive(Debug, Error)]
pub enum FrontEndErrorKind {
    #[error(transparent)]
    SyntaxError(#[from] SyntaxErrorKind),
    #[error(transparent)]
    ParseError(#[from] ParseErrorKind),
    #[error(transparent)]
    SearchsPathError(SearchPathsError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct FrontEndError {
    kind: FrontEndErrorKind,
    severity: Severity,
    pos: std::ops::Range<usize>,
}

impl std::fmt::Display for FrontEndError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(_f, "{}", self.kind)
    }
}

impl std::error::Error for FrontEndError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl<'a> ParseError<Span<'a>> for FrontEndError {
    fn from_error_kind(_input: &Span<'a>, kind: ParseErrorKind, severity: Severity) -> Self {
        let pos = _input.get_range();
        Self {
            kind: kind.into(),
            severity,
            pos,
        }
    }

    fn append(_input: &Span<'a>, _kind: ParseErrorKind, _other: Self) -> Self {
        todo!()
    }

    fn set_severity(self, severity: Severity) -> Self {
        Self {
            severity,
            ..self

        }
    }

    fn severity(&self) -> Severity {
        self.severity
    }
}
