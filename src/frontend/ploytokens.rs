use crate::sources::{FileSpan, SourceFile, SourceOrigin, SourceSpan};

use super::tokens::ParseText;

use logos::Logos;

use super::prelude::*;
use super::tokens::Location;

pub(crate) type Token<'a> = super::tokens::Token<ParseText<'a>>;
pub(crate) type SlimToken = super::tokens::Token<FileSpan>;

////////////////////////////////////////////////////////////////////////////////
impl unraveler::Item for SlimToken {
    type Kind = TokenKind;

    fn get_kind(&self) -> Self::Kind {
        self.kind
    }
}

////////////////////////////////////////////////////////////////////////////////
impl<'a> unraveler::Item for Token<'a> {
    type Kind = TokenKind;

    fn get_kind(&self) -> Self::Kind {
        self.kind
    }
}

impl unraveler::Item for TokenKind {
    type Kind = TokenKind;

    fn get_kind(&self) -> Self::Kind {
        *self
    }
}

impl unraveler::Collection for TokenKind {
    type Item = TokenKind;

    fn at(&self, index: usize) -> Option<&Self::Item> {
        if index > 0 {
            None
        } else {
            Some(self)
        }
    }

    fn length(&self) -> usize {
        1
    }
}

fn to_tokens_kinds(source_file: &SourceFile) -> Vec<(TokenKind, std::ops::Range<usize>)> {
    TokenKind::lexer(&source_file.text)
        .spanned()
        .map(|(tok_res, pos)| match tok_res {
            Ok(kind) => (kind, pos),
            Err(_) => (TokenKind::Error, pos),
        })
        .collect()
}

fn to_tokens(source_file: &SourceFile) -> Vec<Token> {
    to_tokens_kinds(source_file)
        .into_iter()
        .map(|(kind, r)| Token {
            kind,
            location: Location::new(r.start, r.len()),
                extra: ParseText::new(&source_file.text[r]),
        })
        .collect()
}

fn to_slim_tokens(source_file: &SourceFile) -> Vec<SlimToken> {
    to_tokens_kinds(source_file)
        .into_iter()
        .map(|(kind, r)| {

            let file_span = source_file
                .get_file_span_from_offset(r.start)
                .expect("What?");

            SlimToken {
                kind,
                location: Location::new(r.start,r.len()),
                extra: file_span,
            }
        })
        .collect()
}

pub fn tokenize(source_file: &SourceFile) -> Vec<Token> {
    let tokes = to_tokens(source_file);
    tokes
}
