use std::collections::HashMap;

use serde::__private::de::IdentifierDeserializer;
use thin_vec::{thin_vec, ThinVec};

use unraveler::{
    alt, cut, is_a, many0, many1, opt, pair, preceded, sep_pair, tag, tuple, wrapped_cut,
    Collection, Item, ParseError, ParseErrorKind, Severity,
};

use super::prelude::*;

////////////////////////////////////////////////////////////////////////////////
// Helpers
fn parse_kind<K>(input: Span, one_of: K, node_kind: AstNodeKind) -> PResult<ParseNode>
where
    K: Collection,
    <K as Collection>::Item: PartialEq + Copy + Item,
    TokenKind: PartialEq<<<K as Collection>::Item as Item>::Kind>,
{
    let (rest, _matched) = is_a(one_of)(input)?;
    let ret = ParseNode::builder(node_kind, input, rest).build();
    Ok((rest, ret))
}

fn parse_wrapped_collection(
    input: Span,
    open: TokenKind,
    close: TokenKind,
    kind: AstNodeKind,
) -> PResult<ParseNode> {
    let (rest, list) = wrapped_cut(open, many0(parse_atom), close)(input)?;
    let x = ParseNode::builder(kind, input, rest).children(list);
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
    parse_kind(
        input,
        [TokenKind::True, TokenKind::False],
        AstNodeKind::Bool,
    )
}

fn parse_quoted(input: Span) -> PResult<ParseNode> {
    let (rest, atom) = preceded(tag(TokenKind::Quote), parse_quotable)(input)?;
    let node = ParseNode::builder(AstNodeKind::Quoted, input, rest).children(vec![atom]);
    Ok((rest, node.into()))
}

fn parse_null(input: Span) -> PResult<ParseNode> {
    let (rest, _) = tag([TokenKind::OpenBracket, TokenKind::CloseBracket])(input)?;
    Ok((
        rest,
        ParseNode::builder(AstNodeKind::Null, input, rest).into(),
    ))
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

fn match_text<'a>(t: &'a Token<'a>, txt: &str) -> bool {
    t.extra.txt == txt
}

fn parse_string(input: Span) -> PResult<ParseNode> {
    parse_kind(input, TokenKind::QuotedString, AstNodeKind::QuotedString)
}

fn parse_aplication(input: Span) -> PResult<ParseNode> {
    let (rest, list) = wrapped_cut(
        TokenKind::OpenBracket,
        many1(parse_atom),
        TokenKind::CloseBracket,
    )(input)?;
    let node = ParseNode::builder(AstNodeKind::Application, input, rest).children(list);
    Ok((rest, node.into()))
}

////////////////////////////////////////////////////////////////////////////////
fn get_text<'a>(input: Span<'a>, txt: &'a str) -> PResult<'a, Span<'a>> {
    use TokenKind::*;
    let (rest, matched) = tag(Identifier)(input)?;

    if match_text(&matched.span[0], txt) {
        Ok((rest, matched))
    } else {
        Err(FrontEndError::from_error_kind(
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
        tuple((tag(OpenBracket), txt_tag("define"))),
        cut(tuple((
            parse_symbol,
            opt(parse_meta),
            parse_atom,
            tag(CloseBracket),
        ))),
    )(input)?;

    Ok((
        rest,
        ParseNode::builder(Define, input, rest)
            .children([sym, val])
            .meta_opt(meta)
            .into(),
    ))
}

fn parse_if(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::If, TokenKind::*};

    let body = preceded(
        txt_tag("if"),
        cut(tuple((parse_atom, parse_atom, opt(parse_atom)))),
    );

    let (rest, (predicate, is_true, is_false)) =
        wrapped_cut(OpenBracket, body, CloseBracket)(input)?;

    let args: ThinVec<_> = [Some(predicate), Some(is_true), is_false]
        .into_iter()
        .flatten()
        .collect();

    let node = ParseNode::builder(If, input, rest).children(args);

    Ok((rest, node.build()))
}

fn parse_let_arg(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::LetArg, TokenKind::*};
    let (rest, (arg, val)) = pair(parse_arg, parse_atom)(input)?;
    let node = ParseNode::builder(LetArg, input, rest)
        .children([arg, val])
        .build();
    Ok((rest, node))
}

fn parse_let_args(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    let (rest, args) =
        wrapped_cut(OpenSquareBracket, many0(parse_let_arg), CloseSquareBracket)(input)?;
    let node = ParseNode::builder(AstNodeKind::LetArgs, input, rest)
        .children(args)
        .build();
    Ok((rest, node))
}

fn parse_forms(input: Span) -> PResult<Vec<ParseNode>> {
    many0(parse_atom)(input)
}

fn parse_let(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::Let, TokenKind::*};

    let (rest, (args, forms)) = wrapped_cut(
        OpenBracket,
        preceded(txt_tag("let"), cut(pair(parse_let_args, parse_forms))),
        CloseBracket,
    )(input)?;

    let node = ParseNode::builder(Let, input, rest)
        .child(args)
        .children(forms)
        .build();
    Ok((rest, node))
}

fn parse_arg(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::Arg, TokenKind::*};
    parse_kind(input, [Identifier, FqnIdentifier], Arg)
}

fn parse_args(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::Args, TokenKind::*};
    let (rest, args) = wrapped_cut(OpenSquareBracket, many0(parse_arg), CloseSquareBracket)(input)?;
    let node = ParseNode::builder(Args, input, rest).children(args).build();
    Ok((rest, node))
}

fn txt_tag<'a>(txt: &'a str) -> impl Fn(Span<'a>) -> Result<(Span<'a>, Span<'a>), FrontEndError> {
    |i| get_text(i, txt)
}

fn parse_lambda(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::Lambda, TokenKind::*};

    let (rest, (args, forms)) = wrapped_cut(
        OpenBracket,
        preceded(txt_tag("fn"), cut(pair(parse_args, parse_forms))),
        CloseBracket,
    )(input)?;

    let node = ParseNode::builder(Lambda, input, rest)
        .child(args)
        .children(forms)
        .build();
    Ok((rest, node))
}

fn parse_simple<'a>(input: Span<'a>, txt: &'a str, kind: AstNodeKind) -> PResult<'a, ParseNode> {
    use TokenKind::*;
    let (rest, args) = wrapped_cut(
        OpenBracket,
        preceded(txt_tag(txt), parse_forms),
        CloseBracket,
    )(input)?;
    let node = ParseNode::builder(kind, input, rest).children(args).build();
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
    let node = ParseNode::builder(AstNodeKind::Pair, input, rest).children([a, b]);
    Ok((rest, node.into()))
}

fn parse_keyword(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    let (rest, _) = tag(KeyWord)(input)?;
    Ok((
        rest,
        ParseNode::builder(AstNodeKind::KeyWord, input, rest).into(),
    ))
}

fn parse_map(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    let (rest, kids) = wrapped_cut(OpenBrace, many0(parse_pair), CloseBrace)(input)?;
    let node = ParseNode::builder(AstNodeKind::Map, input, rest).children(kids);
    Ok((rest, node.into()))
}

fn parse_meta(input: Span) -> PResult<ParseNode> {
    use {
        AstNodeKind::{KeyWord, Pair},
        TokenKind::Caret,
    };

    let check_pair = |x: &ParseNode| x.is_kind(Pair) && x.children[0].is_kind(KeyWord);

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
    let x = ParseNode::builder(AstNodeKind::Program, input, rest).children(matched);
    Ok((rest, x.into()))
}
