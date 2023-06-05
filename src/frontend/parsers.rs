use std::collections::HashMap;

use thin_vec::{thin_vec, ThinVec};

use unraveler::{
    alt, cut, is_a, many0, many1, opt, pair, preceded, sep_pair, tag, tuple, wrapped, Collection,
    Item, ParseError, ParseErrorKind, Severity,
};

use super::prelude::*;

////////////////////////////////////////////////////////////////////////////////
// Helpers
fn parse_kind<'a, K>(input: Span<'a>, one_of: K, node_kind: AstNodeKind) -> PResult<'a, ParseNode>
where
    K: Collection,
    <K as Collection>::Item: PartialEq + Copy + Item,
    TokenKind: PartialEq<<<K as Collection>::Item as Item>::Kind>,
{
    let (rest, _matched) = is_a(one_of)(input)?;
    let ret = ParseNode::new(node_kind, input, rest).build();
    Ok((rest, ret))
}

fn parse_wrapped_collection(
    input: Span,
    open: TokenKind,
    close: TokenKind,
    kind: AstNodeKind,
) -> PResult<ParseNode> {
    let (rest, list) = wrapped(open, many0(parse_atom), close)(input)?;
    let x = ParseNode::new(kind, input, rest).children(list);
    Ok((rest, x.into()))
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
    let node = ParseNode::new(AstNodeKind::Quoted, input, rest).children(vec![atom]);
    Ok((rest, node.into()))
}

fn parse_null(input: Span) -> PResult<ParseNode> {
    let (rest, _) = tag([TokenKind::OpenBracket, TokenKind::CloseBracket])(input)?;
    Ok((rest, ParseNode::new(AstNodeKind::Null, input, rest).into()))
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
    let node = ParseNode::new(AstNodeKind::Application, input, rest).children(list);
    Ok((rest, node.into()))
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
    use AstNodeKind::Define;
    use TokenKind::*;

    let (rest, (sym, meta, val, _)) = preceded(
        tuple((tag(OpenBracket), |i| parse_text(i, "define"))),
        cut(tuple((
            parse_symbol,
            opt(parse_meta),
            parse_atom,
            tag(CloseBracket),
        ))),
    )(input)?;

    Ok((
        rest,
        ParseNode::new(Define, input, rest)
            .children([sym, val])
            .meta_opt(meta)
            .into(),
    ))
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

    let node = ParseNode::new(AstNodeKind::If, input, rest)
        .children(args)
        .build();
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

    let (rest, (args, forms, _)) = preceded(
        tuple((tag(OpenBracket), |i| parse_text(i, txt))),
        cut(tuple((parse_array, many0(parse_atom), tag(CloseBracket)))),
    )(input)?;

    let node = ParseNode::new(kind, input, rest)
        .child(args)
        .children(forms)
        .build();
    Ok((rest, node))
}

fn parse_simple<'a>(input: Span<'a>, txt: &'a str, kind: AstNodeKind) -> PResult<'a, ParseNode> {
    use TokenKind::*;
    let (rest, (args, _)) = preceded(
        tuple((tag(OpenBracket), |i| parse_text(i, txt))),
        cut(tuple((many1(parse_atom), tag(CloseBracket)))),
    )(input)?;
    let node = ParseNode::new(kind, input, rest).children(args).build();
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
    let node = ParseNode::new(AstNodeKind::Pair, input, rest).children([a, b]);
    Ok((rest, node.into()))
}

fn parse_keyword(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    let (rest, _) = tag(KeyWord)(input)?;
    Ok((
        rest,
        ParseNode::new(AstNodeKind::KeyWord, input, rest).into(),
    ))
}

fn parse_map(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    let (rest, kids) = wrapped(OpenBrace, many0(parse_pair), CloseBrace)(input)?;
    let node = ParseNode::new(AstNodeKind::Map, input, rest).children(kids);
    Ok((rest, node.into()))
}

fn parse_meta(input: Span) -> PResult<ParseNode> {
    use { AstNodeKind::{KeyWord, Pair}, TokenKind::Caret };

    let check_pair =
        |x: &ParseNode| x.is_kind(Pair) && x.children[0].is_kind(KeyWord);

    let (rest, matched) = preceded(tag(Caret), parse_map)(input)?;

    let all_kw_pairs = matched.children.iter().all(check_pair);

    if !all_kw_pairs {
        todo!("type check failure for meta")
    }

    Ok((rest, matched.change_kind(AstNodeKind::MetaData)))
}

fn parse_atom(input: Span) -> PResult<ParseNode> {
    alt((
        parse_keyword,
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

pub(crate) fn parse_program(input: Span) -> PResult<ParseNode> {
    let (rest, matched) = many0(parse_atom)(input)?;
    let x = ParseNode::new(AstNodeKind::Program, input, rest).children(matched);
    Ok((rest, x.into()))
}
