use crate::sources::{SourceOrigin, SourceFile};

use super::tokens::{ FragementLocation,ParseText};
use logos::Logos;

use super::prelude::*;

pub (crate) type Token<'a> = super::tokens::Token<ParseText<'a>>;

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
            location: FragementLocation {
                loc: r.clone().into(),
                extra: ParseText::new(&source_file.text[r]),
            },
        })
        .collect()
}

pub fn tokenize(source_file: &SourceFile) -> Vec<Token> {
    let tokes = to_tokens(source_file);
    tokes
}
