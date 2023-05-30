use thin_vec::{thin_vec, ThinVec};

use unraveler::{
    alt, is_a, many0, pair, preceded, sep_pair, tag, tuple, wrapped, Collection, Item, ParseError,
};

use super::{
    error::{FrontEndError, PResult, PlError},
    ploytokens::Token,
    tokens::{ParseText, TokenKind},
    Span,
};

#[derive(Clone, PartialEq, Debug)]
pub enum AstNodeKind {
    Quoted,
    Lambda,
    Program,
    Array,
    Number(TokenKind),
    List,
    Null,
    Bool(bool),
    QuotedString,
    Symbol,
    InternedSymbol,
    Map,
    Pair,
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

////////////////////////////////////////////////////////////////////////////////

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

fn parse_wrapped_collection(
    input: Span,
    open: TokenKind,
    close: TokenKind,
    kind: AstNodeKind,
) -> PResult<AstNode> {
    let (rest, list) = wrapped(open, many0(parse_atom), close)(input)?;
    let x = AstNode::from_spans(kind, input, rest).with_children(list);
    Ok((rest, x))
}

fn parse_quotable(input: Span) -> PResult<AstNode> {
    alt((parse_list, parse_array, parse_symbol, parse_map))(input)
}

////////////////////////////////////////////////////////////////////////////////
fn parse_number(input: Span) -> PResult<AstNode> {
    use TokenKind::{BinNumber, DecNumber, HexNumber};
    let (rest, _matched) = is_a([DecNumber, HexNumber, BinNumber])(input)?;
    let ret = AstNode::from_spans(AstNodeKind::Number(DecNumber), input, rest);
    Ok((rest, ret))
}

fn parse_bool(input: Span) -> PResult<AstNode> {
    use TokenKind::{False, True};
    let (rest, matched) = is_a([True, False])(input)?;
    let ret = AstNode::from_spans(AstNodeKind::Bool(matched == TokenKind::True), input, rest);
    Ok((rest, ret))
}

pub fn parse_quoted(input: Span) -> PResult<AstNode> {
    let (rest, atom) = preceded(tag(TokenKind::Quote), parse_quotable)(input)?;
    let ret = AstNode::from_spans(AstNodeKind::Quoted, input, rest).with_children(vec![atom]);
    Ok((rest, ret))
}

fn parse_null(input: Span) -> PResult<AstNode> {
    let (rest, _) = tag([TokenKind::OpenBracket, TokenKind::CloseBracket])(input)?;
    Ok((rest, AstNode::from_spans(AstNodeKind::Null, input, rest)))
}

fn parse_symbol(input: Span) -> PResult<AstNode> {
    use TokenKind::{FqnIdentifier, Identifier};
    parse_kind(input, [Identifier, FqnIdentifier], AstNodeKind::Symbol)
}

fn parse_string(input: Span) -> PResult<AstNode> {
    parse_kind(input, TokenKind::QuotedString, AstNodeKind::QuotedString)
}

fn parse_list(input: Span) -> PResult<AstNode> {
    use TokenKind::{CloseBracket, OpenBracket};
    parse_wrapped_collection(input, OpenBracket, CloseBracket, AstNodeKind::List)
}

fn parse_array(input: Span) -> PResult<AstNode> {
    use AstNodeKind::Array;
    use TokenKind::{CloseSquareBracket, OpenSquareBracket};
    parse_wrapped_collection(input, OpenSquareBracket, CloseSquareBracket, Array)
}

fn parse_map(input: Span) -> PResult<AstNode> {
    use TokenKind::*;
    let (rest, list) = wrapped(
        OpenBrace,
        many0(sep_pair(parse_atom, tag(Colon), parse_atom)),
        CloseBrace,
    )(input)?;
    let kids: Vec<_> = list.into_iter().map(|(a, b)| [a, b]).flatten().collect();
    let x = AstNode::from_spans(AstNodeKind::Map, input, rest).with_children(kids);
    Ok((rest, x))
}

fn parse_atom(input: Span) -> PResult<AstNode> {
    alt((
        parse_quoted,
        parse_number,
        parse_string,
        parse_bool,
        parse_list,
        parse_array,
        parse_symbol,
        parse_map,
    ))(input)
}

pub fn parse_program(input: Span) -> PResult<AstNode> {
    let (rest, matched) = many0(parse_atom)(input)?;
    let x = AstNode::from_spans(AstNodeKind::Program, input, rest).with_children(matched);
    Ok((rest, x))
}

////////////////////////////////////////////////////////////////////////////////
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
