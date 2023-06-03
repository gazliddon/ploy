use std::{collections, io::WriterPanicked};
use thin_vec::{thin_vec, ThinVec};

use unraveler::{
    alt, cut, is_a, many0, many1, opt, pair, preceded, sep_pair, tag, tuple, wrapped, Collection,
    Item, ParseError, ParseErrorKind, Parser, Severity,
};

use super::{
    error::FrontEndError,
    tokens::ParseText,
};

use super::prelude::*;

#[derive(Clone, PartialEq, Debug)]
pub (crate) struct ParseNode {
    pub kind: AstNodeKind,
    pub range: std::ops::Range<usize>,
    pub children: ThinVec<ParseNode>,
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
// Helpers

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

// end Helpers
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
    parse_kind(input, [True, False], AstNodeKind::Bool)
}

fn parse_quoted(input: Span) -> PResult<ParseNode> {
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
    parse_kind(input, [Identifier, FqnIdentifier], AstNodeKind::Symbol)
}

fn parse_builtin(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;

    parse_kind(
        input,
        [Star, Plus, Minus, Slash, Equals],
        AstNodeKind::BuiltIn,
    )
}

fn get_text<'a>(t: &'a Token<'a>) -> &'a str {
    t.location.extra.txt
}

fn parse_string(input: Span) -> PResult<ParseNode> {
    parse_kind(input, TokenKind::QuotedString, AstNodeKind::QuotedString)
}

fn parse_aplication(input: Span) -> PResult<ParseNode> {
    let (rest, list) = wrapped(
        TokenKind::OpenBracket,
        many1(parse_atom),
        TokenKind::CloseBracket,
    )(input)?;
    let x = ParseNode::from_spans(AstNodeKind::Application, input, rest).with_children(list);
    Ok((rest, x))
}

////////////////////////////////////////////////////////////////////////////////
fn parse_text<'a>(input: Span<'a>, txt: &'a str) -> PResult<'a, Span<'a>> {
    use TokenKind::*;
    let (rest, matched) = tag(Identifier)(input)?;

    if get_text(&matched.span[0]) == txt {
        Ok((rest, matched))
    } else {
        Err(PlError::from_error_kind(
            &input,
            ParseErrorKind::NoMatch,
            Severity::Error,
        ))
    }
}

fn parse_specials(input: Span) -> PResult<ParseNode> {
    let (rest, matched) = alt((
        parse_if,
        parse_define,
        parse_lambda,
        parse_let,
        parse_and,
        parse_or,
        // parse_cond,
        // parse_do,
        // parse_macro,
    ))(input)?;

    Ok((rest, matched))
}

fn parse_or(input: Span) -> PResult<ParseNode> {
    parse_simple(input, "or", AstNodeKind::Or)
}

fn parse_and(input: Span) -> PResult<ParseNode> {
    parse_simple(input, "and", AstNodeKind::And)
}

fn parse_do(input: Span) -> PResult<ParseNode> {
    parse_simple(input, "do", AstNodeKind::Do)
}
fn parse_cond(input: Span) -> PResult<ParseNode> {
    parse_simple(input, "cond", AstNodeKind::Cond)
}

fn parse_macro(input: Span) -> PResult<ParseNode> {
    parse_simple(input, "macro", AstNodeKind::Macro)
}

fn parse_define(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;

    let (rest, (sym, val, _)) = preceded(
        tuple((tag(OpenBracket), |i| parse_text(i, "define"))),
        cut(tuple((parse_symbol, parse_atom, tag(CloseBracket)))),
    )(input)?;
    let node = ParseNode::from_spans(AstNodeKind::Define, input, rest).with_children([sym, val]);
    Ok((rest, node))
}

fn parse_if(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;

    let (rest, (p, is_true, is_false, _)) = preceded(
        tuple((tag(OpenBracket), |i| parse_text(i, "if"))),
        cut(tuple((
            parse_atom,
            parse_atom,
            opt(parse_atom),
            tag(CloseBracket),
        ))),
    )(input)?;

    let args: ThinVec<_> = [Some(p), Some(is_true), is_false]
        .into_iter()
        .filter_map(|x| x)
        .collect();
    let node = ParseNode::from_spans(AstNodeKind::If, input, rest).with_children(args);
    Ok((rest, node))
}

fn parse_let(input: Span) -> PResult<ParseNode> {
    parse_let_lambda(input, "let", AstNodeKind::Let)
}

fn parse_lambda(input: Span) -> PResult<ParseNode> {
    parse_let_lambda(input, "fn", AstNodeKind::Lambda)
}

fn parse_let_lambda<'a>(
    input: Span<'a>,
    txt: &'a str,
    kind: AstNodeKind,
) -> PResult<'a, ParseNode> {
    use TokenKind::*;

    let (rest, (args,forms, _)) = preceded(
        tuple((tag(OpenBracket), |i| parse_text(i,txt))),
        cut(tuple((
            parse_array,
            many0(parse_atom),
            tag(CloseBracket),
        ))),
    )(input)?;

    let mut args = vec![args];
    args.extend(forms.into_iter());
    let node = ParseNode::from_spans(kind, input, rest).with_children(args);
    Ok((rest, node))
}

fn parse_simple<'a>(input: Span<'a>, txt: &'a str, kind: AstNodeKind) -> PResult<'a, ParseNode> {
    use TokenKind::*;
    let (rest, (args, _)) = preceded(
        tuple((tag(OpenBracket), |i| parse_text(i,txt))),
        cut(tuple(( many1( parse_atom ), tag(CloseBracket)))),
    )(input)?;
    let node = ParseNode::from_spans(kind, input, rest).with_children(args);
    Ok((rest, node))
}

////////////////////////////////////////////////////////////////////////////////

fn parse_list(input: Span) -> PResult<ParseNode> {
    use TokenKind::{CloseBracket, OpenBracket, Quote};
    let (rest, _) = tag(Quote)(input)?;
    parse_wrapped_collection(rest, OpenBracket, CloseBracket, AstNodeKind::List)
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
        parse_number,
        parse_string,
        parse_bool,
        parse_builtin,
        parse_symbol,
        parse_specials,
        parse_aplication,
        parse_array,
        parse_list,
        parse_quoted,
        parse_map,
    ))(input)
}

pub (crate) fn parse_program(input: Span) -> PResult<ParseNode> {
    let (rest, matched) = many0(parse_atom)(input)?;
    let x = ParseNode::from_spans(AstNodeKind::Program, input, rest).with_children(matched);
    Ok((rest, x))
}
