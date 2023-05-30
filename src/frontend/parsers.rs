use thin_vec::{thin_vec, ThinVec};
use unraveler::{
    alt, is_a, many0, many1,pair, preceded, sep_pair, tag, tuple, wrapped, Collection, Item, ParseError,
};

use super::{
    error::{FrontEndError, PResult, PlError},
    ploytokens::Token,
    tokens::{ParseText, TokenKind},
    ast::AstNodeKind,
    Span,
};

#[derive(Clone, PartialEq, Debug)]
pub struct ParseNode {
    kind: AstNodeKind,
    range: std::ops::Range<usize>,
    children: ThinVec<ParseNode>,
}

impl ParseNode {
    pub fn new(kind: AstNodeKind, start: usize, len: usize) -> Self {
        Self {
            kind,
            range: start..start + len,
            children: thin_vec![],
        }
    }
    pub fn from_spans(kind: AstNodeKind, input: Span, rest: Span) -> Self {
        let input = input.get_range();
        let rest = rest.get_range();

        Self {
            kind,
            range: input.start..rest.start,
            children: thin_vec![],
        }
    }

    pub fn with_children<X: Into<ThinVec<ParseNode>>>(mut self, children: X) -> Self {
        self.children = children.into();
        self
    }
}

////////////////////////////////////////////////////////////////////////////////
fn parse_kind<'a, K>(input: Span<'a>, one_of: K, node_kind: AstNodeKind) -> PResult<'a, ParseNode>
where
    K: Collection,
    <K as Collection>::Item: PartialEq + Copy + Item,
    TokenKind: PartialEq<<<K as Collection>::Item as Item>::Kind>,
{
    let (rest, _matched) = is_a(one_of)(input)?;
    let ret = ParseNode::from_spans(node_kind, input, rest);
    Ok((rest, ret))
}

fn parse_wrapped_collection(
    input: Span,
    open: TokenKind,
    close: TokenKind,
    kind: AstNodeKind,
) -> PResult<ParseNode> {
    let (rest, list) = wrapped(open, many0(parse_atom), close)(input)?;
    let x = ParseNode::from_spans(kind, input, rest).with_children(list);
    Ok((rest, x))
}

fn parse_quotable(input: Span) -> PResult<ParseNode> {
    alt((parse_array, parse_symbol, parse_map))(input)
}

////////////////////////////////////////////////////////////////////////////////
fn parse_number(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;

    parse_kind(
        input,
        [DecNumber, HexNumber, BinNumber],
        AstNodeKind::Number,
    )
}

fn parse_bool(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    parse_kind(
        input,
        [True, False],
        AstNodeKind::Bool,
    )
}

pub fn parse_quoted(input: Span) -> PResult<ParseNode> {
    let (rest, atom) = preceded(tag(TokenKind::Quote), parse_quotable)(input)?;
    let ret = ParseNode::from_spans(AstNodeKind::Quoted, input, rest).with_children(vec![atom]);
    Ok((rest, ret))
}

fn parse_null(input: Span) -> PResult<ParseNode> {
    let (rest, _) = tag([TokenKind::OpenBracket, TokenKind::CloseBracket])(input)?;
    Ok((rest, ParseNode::from_spans(AstNodeKind::Null, input, rest)))
}

fn parse_symbol(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    parse_kind(
        input,
        [Identifier, FqnIdentifier, ],
        AstNodeKind::Symbol,
    )
}

fn parse_special(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;

    parse_kind(
        input,
        [Star, Plus, Minus, Slash, Equals ],
        AstNodeKind::Special,
    )
}

fn parse_string(input: Span) -> PResult<ParseNode> {
    parse_kind(input, TokenKind::QuotedString, AstNodeKind::QuotedString)
}

fn parse_list(input: Span) -> PResult<ParseNode> {
    use TokenKind::{CloseBracket, OpenBracket,Quote};
    let (rest,_) = tag(Quote)(input)?;
    parse_wrapped_collection(rest, OpenBracket, CloseBracket, AstNodeKind::List)
}

fn parse_aplication(input: Span) -> PResult<ParseNode> {
    use TokenKind::{CloseBracket, OpenBracket};
    let (rest, list) = wrapped(OpenBracket, many1(parse_atom), CloseBracket)(input)?;
    let x = ParseNode::from_spans(AstNodeKind::Application, input, rest).with_children(list);
    Ok((rest, x))

}

fn parse_array(input: Span) -> PResult<ParseNode> {
    use AstNodeKind::Array;
    use TokenKind::{CloseSquareBracket, OpenSquareBracket};
    parse_wrapped_collection(input, OpenSquareBracket, CloseSquareBracket, Array)
}

fn parse_pair(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    let (rest, (a, b)) = sep_pair(parse_atom, tag(Colon), parse_atom)(input)?;
    let node = ParseNode::from_spans(AstNodeKind::Pair, input, rest).with_children([a, b]);
    Ok((rest, node))
}

fn parse_map(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    let (rest, kids) = wrapped(OpenBrace, many0(parse_pair), CloseBrace)(input)?;
    let node = ParseNode::from_spans(AstNodeKind::Map, input, rest).with_children(kids);
    Ok((rest, node))
}

fn parse_atom(input: Span) -> PResult<ParseNode> {
    alt((
        parse_list,
        parse_quoted,
        parse_aplication,
        parse_number,
        parse_string,
        parse_bool,
        parse_array,
        parse_special,
        parse_symbol,
        parse_map,
    ))(input)
}

pub fn parse_program(input: Span) -> PResult<ParseNode> {
    let (rest, matched) = many0(parse_atom)(input)?;
    let x = ParseNode::from_spans(AstNodeKind::Program, input, rest).with_children(matched);
    Ok((rest, x))
}

