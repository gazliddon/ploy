use crate::span::Span;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ParseErrorKind {
    TookTooMany,
    SkippedTooMany,
    IllegalSplitIndex,
    NoMatch,
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



