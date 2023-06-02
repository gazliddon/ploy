use crate::error::{PResult, ParseError, ParseErrorKind, Severity};
use crate::traits::*;

pub trait Item: Copy {
    type Kind: Copy + Clone + PartialEq;

    fn is_same_kind<I>(&self, other: &I) -> bool
    where
        I: Item<Kind = Self::Kind>,
    {
        self.get_kind() == other.get_kind()
    }

    fn is_kind(&self, k: Self::Kind) -> bool {
        self.get_kind() == k
    }

    fn get_kind(&self) -> Self::Kind;
}

// impl<X> Item for X
//     where X: Copy + PartialEq
// {
//     type Kind = X;

//     fn get_kind(&self) -> Self::Kind {
//         *self
//     }
// }

#[derive(Copy, Clone, Debug)]
pub struct Span<'a, I>
where
    I: Item,
{
    pub position: usize, // index into parent doc
    pub span: &'a [I],   // this span
}

impl<'a, I> Span<'a, I>
where
    I: Item,
{
    pub fn get_range(&self) -> std::ops::Range<usize> {
        self.position..self.position + self.len()
    }

    pub fn from_slice(text: &'a [I]) -> Self {
        Self::new(0, text)
    }

    pub fn as_slice(&self) -> &[I] {
        self.span
    }

    pub fn new(position: usize, span: &'a [I]) -> Self {
        Self { position, span }
    }

    pub fn len(&self) -> usize {
        self.span.len()
    }

    pub fn is_empty(&self) -> bool {
        self.span.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &I> + '_ {
        self.span.iter()
    }

    pub fn kinds_iter(&self) -> impl Iterator<Item = I::Kind> + '_ {
        self.span.iter().map(|i| i.get_kind())
    }

    pub fn take(&self, n: usize) -> Result<Self, ParseErrorKind> {
        if n > self.len() {
            Err(ParseErrorKind::TookTooMany.into())
        } else {
            Ok(Self::new(self.position, &self.span[..n]))
        }
    }

    pub fn drop(&self, n: usize) -> Result<Self, ParseErrorKind> {
        if n > self.len() {
            Err(ParseErrorKind::SkippedTooMany.into())
        } else {
            Ok(Self::new(self.position + 1, &self.span[n..]))
        }
    }

    pub fn split(&self, n: usize) -> Result<(Self, Self), ParseErrorKind> {
        if n > self.len() {
            Err(ParseErrorKind::IllegalSplitIndex.into())
        } else {
            Ok((self.drop(n)?, self.take(n)?))
        }
    }

    fn match_token(&'a self, other: &'a [<I as Item>::Kind]) -> PResult<'a, I> {
        if self.len() < other.len() {
            Err(ParseErrorKind::NoMatch)
        } else {
            let it = self.iter().zip(other.iter());

            for (i, k) in it {
                if i.get_kind() != *k {
                    return Err(ParseErrorKind::NoMatch);
                }
            }

            self.split(other.len())
        }
    }

    fn match_kind(&self, k: I::Kind) -> bool {
        self.span
            .get(0)
            .map(|i| i.is_kind(i.get_kind()))
            .unwrap_or(false)
    }
}

impl<'a, I, E> Splitter<E> for Span<'a, I>
where
    I: Item,
    E: ParseError<Span<'a, I>>,
{
    fn split_at(&self, pos: usize) -> Result<(Self, Self), E> {
        self.split(pos)
            .map_err(|e| ParseError::from_error_kind(self, e, Severity::Error))
    }
}

impl<I> Collection for Span<'_, I>
where
    I: Item,
{
    type Item = I;

    fn at<'a>(&'a self, index: usize) -> Option<&'a Self::Item> {
        self.span.get(index)
    }

    fn length(&self) -> usize {
        self.span.len()
    }
}

// impl Item for char {
//     type Kind = char;

//     fn get_kind(&self) -> Self::Kind {
//         *self
//     }
// }

// impl Item for u8 {
//     type Kind = u8;
//     fn get_kind(&self) -> Self::Kind {
//         *self
//     }
// }
