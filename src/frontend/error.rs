use super::prelude::*;
use super::syntax::SyntaxErrorKind;
use crate::sources::{SearchPathsError, FileSpan};
use thiserror::Error;
use unraveler::{ParseError, ParseErrorKind, Severity};

pub type PResult<'a, O, E = FrontEndError> = Result<(Span<'a>, O), E>;

#[derive(Debug, Error, Clone)]
pub enum FrontEndErrorKind {
    #[error(transparent)]
    SyntaxError(#[from] SyntaxErrorKind),
    #[error(transparent)]
    ParseError(#[from] ParseErrorKind),
    #[error(transparent)]
    SearchsPathError(SearchPathsError),
    #[error("{0}")]
    Other(String),
}

#[derive(Clone,Debug)]
enum ErrorPos {
    TokenRange(std::ops::Range<usize>),
    FileSpan(FileSpan)
}

#[derive(Debug, Clone)]
pub struct FrontEndError {
    pub kind: FrontEndErrorKind,
    pub severity: Severity,
    pub pos: std::ops::Range<usize>,
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

impl FrontEndError {
    pub fn set_kind<K: Into<FrontEndErrorKind>>(self, kind: K) -> Self {
        Self {
            kind: kind.into(),
            ..self
        }
    }
}

impl<'a> ParseError<Span<'a>> for FrontEndError {
    fn from_error_kind(input: Span<'a>, kind: ParseErrorKind, severity: Severity) -> Self {
        let pos =  if input.len() == 0 {
            0..0
        } else {
            let start = input.as_slice().first().unwrap().extra.as_range();
            let end = input.as_slice().last().unwrap().extra.as_range();
            let start_t = start.start;
            let end_t = end.end;
            start_t .. end_t
        };

        Self {
            kind: kind.into(),
            severity,
            pos,
        }
    }

    fn append(_input: Span<'a>, _kind: ParseErrorKind, _other: Self) -> Self {
        todo!()
    }

    fn set_severity(self, severity: Severity) -> Self {
        Self { severity, ..self }
    }

    fn change_kind(self, kind: ParseErrorKind) -> Self {
        Self {
            kind: kind.into(),
            ..self
        }
    }

    fn severity(&self) -> Severity {
        self.severity
    }
}
