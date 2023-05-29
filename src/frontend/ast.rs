use std::fmt::Binary;
use std::iter::{Copied, Enumerate};

use unraveler::{alt, pair, tag};
use unraveler::{many0, ParseError};

use super::{
    error::FrontEndError,
    ploytokens::Token,
    tokens::{ParseText, TokenKind},
};

pub type Span<'a> = unraveler::Span<'a, Token<'a>>;

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

type PResult<'a, O, E = PlError> = Result<(Span<'a>, O), E>;

pub struct PlError {}

impl<'a> unraveler::ParseError<Span<'a>> for PlError {
    fn from_error_kind(_input: &Span<'a>, _kind: unraveler::ParseErrorKind) -> Self {
        todo!()
    }

    fn append(_input: &Span<'a>, _kind: unraveler::ParseErrorKind, _other: Self) -> Self {
        todo!()
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AstNodeKind {
    Symbol,
    Number(TokenKind),
    List,
    Null,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct AstNode {
    kind: AstNodeKind,
}

impl AstNode {
    pub fn new(_kind: AstNodeKind, _span: Span) -> Self {
        panic!()
    }
}

pub fn parse_number(input: Span) -> PResult<AstNode> {
    use unraveler::is_a;
    use TokenKind::*;
    let (rest, _matched) = is_a([DecNumber, HexNumber, BinNumber])(input)?;
    let ret = AstNode::new(AstNodeKind::Number(DecNumber), rest);
    Ok((rest, ret))
}

pub fn parse_string(_input: Span) -> PResult<AstNode> {
    panic!()
}

pub fn parse_bool(_input: Span) -> PResult<AstNode> {
    panic!()
}

pub fn parse_symbol(_input: Span) -> PResult<AstNode> {
    panic!()
}

pub fn parse_null(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let (rest, _) = pair(tag([OpenBracket]), tag([CloseBracket]))(input)?;
    Ok((rest, AstNode::new(AstNodeKind::Null, input)))
}

pub fn parse_atom(_input: Span) -> PResult<AstNode> {
    use TokenKind::*;

    let (rest, matched) = alt((
        parse_number,
        parse_string,
        parse_bool,
        parse_symbol,
        parse_null,
        parse_list,
    ))(_input)?;

    Ok((rest, matched))
}

pub fn parse_list(_input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let rest = _input;

    let (rest, _) = tag([OpenBracket])(rest)?;
    let (rest, _list) = many0(parse_atom)(rest)?;
    let (rest, _) = tag([CloseBracket])(rest)?;

    let x = AstNode::new(AstNodeKind::List, _input);

    Ok((rest, x))
}

pub struct Ast {
    source: String,
}

pub fn to_ast(_tokes: Vec<Token>) -> Ast {
    panic!()
}
