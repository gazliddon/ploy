use crate::span::Span;
use thiserror::Error;

#[derive(Error,Copy, Clone, Debug, PartialEq, )]
pub enum ParseErrorKind {
    #[error("not enough to take")]
    TookTooMany,
    #[error("not enough to skip")]
    SkippedTooMany,
    #[error("not enough to split")]
    IllegalSplitIndex,
    #[error("No match")]
    NoMatch,
    #[error("Needed one or more matches")]
    NeededOneOrMore,
}

pub type PResult<'a, I, O = Span<'a, I>> = Result<(Span<'a, I>, O), ParseErrorKind>;

pub enum Severity {
    Normal,
    Fatal,
}

pub trait ParseError<I>: Sized {
    fn from_error_kind(input: &I, kind: ParseErrorKind) -> Self;
    fn append(input: &I, kind: ParseErrorKind, other: Self) -> Self;
    fn is_fatal(&self) -> bool {
        false
    }
}



