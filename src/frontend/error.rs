use thiserror::Error;
use super::Span;

use unraveler::{ ParseError, ParseErrorKind };

#[derive(Debug,Error,)]
pub enum FrontEndError {
    #[error(transparent)]
    ParseError(#[from] ParseErrorKind),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug,)]
pub struct PlError {
    kind : FrontEndError,
}

pub type PResult<'a, O, E = PlError> = Result<(Span<'a>, O), E>;

impl<'a> ParseError<Span<'a>> for PlError {
    fn from_error_kind(_input: &Span<'a>, _kind: ParseErrorKind) -> Self {
        
        let r= Self {
            kind : _kind.into()
        };
        r
    }

    fn append(_input: &Span<'a>, _kind: ParseErrorKind, _other: Self) -> Self {
        todo!()
    }
}
