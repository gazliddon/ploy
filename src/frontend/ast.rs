use unraveler::{alt, is_a, many0, pair, tag, tuple};

use super::{
    error::{FrontEndError, PResult, PlError},
    ploytokens::Token,
    tokens::{ParseText, TokenKind},
    Span,
};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AstNodeKind {
    Symbol,
    Number(TokenKind),
    List,
    Null,
    Bool(bool),
    QuotedString,
}

#[derive(Clone, PartialEq, Debug)]
pub struct AstNode {
    kind: AstNodeKind,
    start: usize,
    len: usize,
}

impl AstNode {
    pub fn new(kind: AstNodeKind, start: usize ,len: usize) -> Self {
        Self {
            kind,
            start,
            len
        }
    }
    pub fn from_spans(kind: AstNodeKind, input: Span, rest: Span) -> Self {
        let input = input.get_range();
        let rest =  rest.get_range();
        Self {
            kind,
            start: input.start,
            len: input.start + (rest.start - input.start)
        }
    }
}

pub fn parse_number(input: Span) -> PResult<AstNode> {
    use unraveler::is_a;
    use TokenKind::*;
    let (rest, _matched) = is_a([DecNumber, HexNumber, BinNumber])(input)?;
    let ret = AstNode::from_spans(AstNodeKind::Number(DecNumber), input,rest);
    Ok((rest, ret))
}

pub fn parse_bool(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let (rest, matched) = is_a([True, False])(input)?;
    let ret = AstNode::from_spans(AstNodeKind::Bool(matched == True), input,rest);
    Ok((rest, ret))
}

fn parse_single<'a>(
    input: Span<'a>,
    is: &[TokenKind],
    node_kind: AstNodeKind,
) -> PResult<'a, AstNode> {
    let (rest, _matched) = is_a(is)(input)?;
    let ret = AstNode::from_spans(node_kind, input,rest);
    Ok((rest, ret))
}

pub fn parse_null(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let (rest, _) = pair(tag([OpenBracket]), tag([CloseBracket]))(input)?;
    Ok((rest, AstNode::from_spans(AstNodeKind::Null, input, rest)))
}

pub fn parse_symbol(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    parse_single(input, &[Identifier, FqnIdentifier], AstNodeKind::Symbol)
}

pub fn parse_string(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    parse_single(input, &[QuotedString], AstNodeKind::QuotedString)
}

pub fn parse_list(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let (rest, (_, _list, _)) =
        tuple((tag([OpenBracket]), many0(parse_atom), tag([CloseBracket])))(input)?;
    let x = AstNode::from_spans(AstNodeKind::List, input, rest);
    Ok((rest, x))
}

pub fn parse_atom(input: Span) -> PResult<AstNode> {
    use TokenKind::*;

    let (rest, matched) = alt((
        parse_number,
        parse_string,
        parse_bool,
        parse_symbol,
        parse_null,
        parse_list,
    ))(input)?;

    Ok((rest, matched))
}

pub struct Ast {
    source: String,
}

pub fn to_ast(_tokes: Vec<Token>) -> Ast {
    panic!()
}
