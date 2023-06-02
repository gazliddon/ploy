use std::fmt::Display;

use thiserror::Error;
use crate::SyntaxErrorKind;

use super::Span;

use unraveler::{ ParseError, ParseErrorKind,Severity };

#[derive(Debug,Error,)]
pub enum FrontEndError {
    #[error(transparent)]
    SyntaxError(#[from] SyntaxErrorKind),
    #[error(transparent)]
    ParseError(#[from] ParseErrorKind),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug,)]
pub struct PlError {
    kind : FrontEndError,
    severity: Severity,
}

impl Display for PlError {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}


impl std::error::Error  for PlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub type PResult<'a, O, E = PlError> = Result<(Span<'a>, O), E>;

impl<'a> ParseError<Span<'a>> for PlError {
    fn from_error_kind(_input: &Span<'a>, kind: ParseErrorKind, severity: Severity) -> Self {
        let r= Self {
            kind : kind.into(),
            severity
        };
        r
    }

    fn append(_input: &Span<'a>, _kind: ParseErrorKind, _other: Self) -> Self {
        todo!()
    }

    fn set_severity(&mut self, sev: Severity) {
        self.severity = sev
    }

    fn severity(&self) -> Severity {
        self.severity
    }
}
