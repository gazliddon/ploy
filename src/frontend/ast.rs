use thin_vec::{thin_vec, ThinVec};

use unraveler::{alt, is_a, many0, pair, tag, tuple, wrapped, Collection, Item, ParseError};

use super::{
    error::{FrontEndError, PResult, PlError},
    ploytokens::Token,
    tokens::{ParseText, TokenKind},
    Span,
};

#[derive(Clone, PartialEq, Debug)]
pub enum AstNodeKind {
    Program,
    Array,
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
    children: ThinVec<AstNode>,
}

impl AstNode {
    pub fn new(kind: AstNodeKind, start: usize, len: usize) -> Self {
        Self {
            kind,
            start,
            len,
            children: thin_vec![],
        }
    }
    pub fn from_spans(kind: AstNodeKind, input: Span, rest: Span) -> Self {
        let input = input.get_range();
        let rest = rest.get_range();
        Self {
            kind,
            start: input.start,
            len: input.start + (rest.start - input.start),
            children: thin_vec![],
        }
    }
    pub fn with_children(mut self, children: Vec<AstNode>) -> Self {
        self.children = children.into();
        self
    }
}

pub fn parse_number(input: Span) -> PResult<AstNode> {
    use unraveler::is_a;
    use TokenKind::*;
    let (rest, _matched) = is_a([DecNumber, HexNumber, BinNumber])(input)?;
    let ret = AstNode::from_spans(AstNodeKind::Number(DecNumber), input, rest);
    Ok((rest, ret))
}

pub fn parse_bool(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let (rest, matched) = is_a([True, False])(input)?;
    let ret = AstNode::from_spans(AstNodeKind::Bool(matched == True), input, rest);
    Ok((rest, ret))
}

fn parse_kind<'a, K>(input: Span<'a>, is: K, node_kind: AstNodeKind) -> PResult<'a, AstNode>
where
    K: Collection,
    <K as Collection>::Item: PartialEq + Copy + Item,
    TokenKind: PartialEq<<<K as Collection>::Item as Item>::Kind>,
{
    let (rest, _matched) = is_a(is)(input)?;
    let ret = AstNode::from_spans(node_kind, input, rest);
    Ok((rest, ret))
}

pub fn parse_null(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let (rest, _) = pair(tag(OpenBracket), tag(CloseBracket))(input)?;
    Ok((rest, AstNode::from_spans(AstNodeKind::Null, input, rest)))
}

pub fn parse_symbol(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    parse_kind(input, [Identifier, FqnIdentifier], AstNodeKind::Symbol)
}

pub fn parse_string(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    parse_kind(input, QuotedString, AstNodeKind::QuotedString)
}

pub fn parse_list(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let (rest, list) = wrapped(OpenBracket, many0(parse_atom), CloseBracket)(input)?;
    let x = AstNode::from_spans(AstNodeKind::List, input, rest).with_children(list);
    Ok((rest, x))
}

pub fn parse_array(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let (rest, list) = wrapped(OpenSquareBracket, many0(parse_atom), CloseSquareBracket)(input)?;
    let x = AstNode::from_spans(AstNodeKind::Array, input, rest).with_children(list);
    Ok((rest, x))
}

pub fn parse_atom(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let (rest, matched) = alt((
        parse_symbol,
        parse_number,
        parse_string,
        parse_bool,
        parse_list,
        parse_array,
    ))(input)?;

    Ok((rest, matched))
}

pub fn parse_program(input: Span) -> PResult<AstNode> {
    let (rest, matched) = many0(parse_list)(input)?;
    let x = AstNode::from_spans(AstNodeKind::Program, input, rest).with_children(matched);
    Ok((rest, x))
}

pub struct Ast {
    source: String,
}

pub fn to_ast<'a>(tokes: &'a Vec<Token>) -> PResult<'a, AstNode> {
    let tokens = Span::new(0, tokes);
    let (rest, matched) = parse_program(tokens)?;
    Ok((rest, matched))
}

mod test {
    use super::*;
    fn test() {}
}
