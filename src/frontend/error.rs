use thiserror::Error;
use super::Span;

use unraveler::{ ParseError, ParseErrorKind };

#[derive(Debug,Error)]
pub enum FrontEndError {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub struct PlError {
    kind : FrontEndError,
}

pub type PResult<'a, O, E = PlError> = Result<(Span<'a>, O), E>;

impl<'a> ParseError<Span<'a>> for PlError {
    fn from_error_kind(_input: &Span<'a>, _kind: ParseErrorKind) -> Self {
        todo!()
    }

    fn append(_input: &Span<'a>, _kind: ParseErrorKind, _other: Self) -> Self {
        todo!()
    }
}
