use std::collections::HashMap;
use thin_vec::{thin_vec, ThinVec};

use unraveler::{
    all, alt, cut, is_a, many0, many1, many_until, not, opt, pair, preceded, sep_pair, succeeded,
    tag, tuple, until, wrapped_cut, Collection, Item, ParseError, ParseErrorKind, Parser, Severity,
};

use crate::frontend::syntax::SyntaxErrorKind;
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
    use {AstNodeKind::*, TokenKind::*};
    let (rest, _) = tag([OpenBracket, CloseBracket])(input)?;
    let node = ParseNode::builder(Null, input, rest);
    Ok((rest, node.build()))
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
    t.extra.get_text() == txt
}

fn parse_string(input: Span) -> PResult<ParseNode> {
    parse_kind(input, TokenKind::QuotedString, AstNodeKind::QuotedString)
}

fn parse_lambda_type(_input: Span) -> PResult<Type> {
    panic!()
}

fn parse_simple_type(input: Span) -> PResult<Type> {
    let (rest, matched) = tag(TokenKind::Identifier)(input)?;
    let text = matched.get(0).unwrap().extra.get_text();

    let ta = match text {
        "bool" => Type::Bool,
        "float" => Type::Float,
        "int" => Type::Integer,
        "string" => Type::String,
        "char" => Type::Char,
        _ => {
            return Err(FrontEndError::from_error_kind(
                input,
                ParseErrorKind::NoMatch,
                Severity::Error,
            ))
        }
    };
    Ok((rest, ta))
}

fn parse_unknown_type(input: Span) -> PResult<Type> {
    let (rest, matched) = tag(TokenKind::Identifier)(input)?;
    let text = matched.get(0).unwrap().extra.get_text();
    Ok((rest, Type::User(text.to_owned())))
}

fn parse_type_annotation(input: Span) -> PResult<Type> {
    use TokenKind::*;
    let (rest, _) = tag(Colon)(input)?;
    let (rest, matched) = alt((parse_simple_type, parse_unknown_type))(rest)?;
    Ok((rest, matched))
}

fn parse_application(input: Span) -> PResult<ParseNode> {
    let body = pair(
        alt((parse_symbol, parse_application, parse_builtin)),
        many0(parse_atom),
    );

    let (rest, (app, forms)) = parse_bracketed(body)(input)?;

    let node = ParseNode::builder(AstNodeKind::Application, input, rest)
        .child(app)
        .children(forms);
    Ok((rest, node.into()))
}

////////////////////////////////////////////////////////////////////////////////
fn get_text<'a>(input: Span<'a>, txt: &'a str) -> PResult<'a, Span<'a>> {
    use TokenKind::*;
    let (rest, matched) = tag(Identifier)(input)?;

    if match_text(&matched.as_slice()[0], txt) {
        Ok((rest, matched))
    } else {
        Err(FrontEndError::from_error_kind(
            input,
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

pub fn parse_define(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::Define, TokenKind::*};

    let body = preceded(
        alt((txt_tag("define"), txt_tag("def"))),
        tuple((parse_arg, parse_atom)),
    );

    let (rest, (sym, val)) = parse_bracketed(body)(input)?;

    Ok((
        rest,
        ParseNode::builder(Define, input, rest)
            .children([sym, val])
            .into(),
    ))
}

pub fn parse_if(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::If, TokenKind::*};

    let body = preceded(
        txt_tag("if"),
        cut(tuple((parse_atom, parse_atom, opt(parse_atom)))),
    );

    let (rest, (predicate, is_true, is_false)) = parse_bracketed(body)(input)?;

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

    let (rest, args) = parse_sq_bracketed(many0(parse_let_arg))(input)?;

    let node = ParseNode::builder(AstNodeKind::LetArgs, input, rest)
        .children(args)
        .build();
    Ok((rest, node))
}

fn parse_forms(input: Span) -> PResult<Vec<ParseNode>> {
    many0(parse_atom)(input)
}

fn wrap_err(
    err: SyntaxErrorKind,
    mut p: impl FnMut(Span) -> Result<(Span, ParseNode), FrontEndError>,
) -> impl FnMut(Span) -> Result<(Span, ParseNode), FrontEndError>
where
{
    move |i| p(i).map_err(|e| e.set_kind(err.clone()))
}

fn txt_tag<'a>(txt: &'a str) -> impl Fn(Span<'a>) -> Result<(Span<'a>, Span<'a>), FrontEndError> {
    |i| get_text(i, txt)
}

pub fn parse_braced<'a, P, O>(
    mut p: P,
) -> impl FnMut(Span<'a>) -> Result<(Span<'a>, O), FrontEndError>
where
    P: Parser<Span<'a>, O, FrontEndError>,
{
    use {SyntaxErrorKind::Expected, TokenKind::*};
    let open = OpenBrace;
    let close = CloseBrace;
    let expected = "closing brace '}'";

    move |input: Span<'a>| {
        let (rest, _) = tag(open)(input)?;
        let (rest, matched) = p.parse(rest)?;
        let (rest, _) = cut(tag(close))(rest)
            .map_err(|e: FrontEndError| e.set_kind(Expected(expected.to_owned())))?;

        Ok((rest, matched))
    }
}

pub fn parse_wrapped_many<'a, P, O>(
    open: TokenKind,
    close: TokenKind,
    expected: &str,
    mut p: P,
) -> impl FnMut(Span<'a>) -> Result<(Span<'a>, Vec<O>), FrontEndError>
where
    P: Parser<Span<'a>, O, FrontEndError>,
{
    use {SyntaxErrorKind::Expected, TokenKind::*};
    let expected = format!("closing '{expected}'");

    move |input: Span<'a>| {
        let pp = |i| p.parse(i);
        let (rest, _) = tag(open)(input)?;
        let (rest, matched) = many_until(pp, tag(close))(rest)?;
        let (rest, _) = cut(tag(close))(rest)
            .map_err(|e: FrontEndError| e.set_kind(Expected(expected.to_owned())))?;
        Ok((rest, matched))
    }
}

pub fn parse_sq_bracketed<'a, P, O>(
    mut p: P,
) -> impl FnMut(Span<'a>) -> Result<(Span<'a>, O), FrontEndError>
where
    P: Parser<Span<'a>, O, FrontEndError>,
{
    use {SyntaxErrorKind::Expected, TokenKind::*};
    let open = OpenSquareBracket;
    let close = CloseSquareBracket;
    let expected = "closing square bracket ']'";

    move |input: Span<'a>| {
        let (rest, _) = tag(open)(input)?;
        let (rest, matched) = p.parse(rest)?;
        let (rest, _) = cut(tag(close))(rest)
            .map_err(|e: FrontEndError| e.set_kind(Expected(expected.to_owned())))?;
        Ok((rest, matched))
    }
}

pub fn parse_bracketed<'a, P, O>(
    mut p: P,
) -> impl FnMut(Span<'a>) -> Result<(Span<'a>, O), FrontEndError>
where
    P: Parser<Span<'a>, O, FrontEndError>,
{
    use {SyntaxErrorKind::Expected, TokenKind::*};

    move |input: Span<'a>| {
        let (rest, _) = tag(OpenBracket)(input)?;
        let (rest, matched) = p.parse(rest)?;
        let (rest, _) = cut(tag(CloseBracket))(rest)
            .map_err(|e: FrontEndError| e.set_kind(Expected("closing bracket ')'".to_owned())))?;

        Ok((rest, matched))
    }
}

fn parse_simple<'a>(input: Span<'a>, txt: &'a str, kind: AstNodeKind) -> PResult<'a, ParseNode> {
    use TokenKind::*;
    let (rest, args) = parse_bracketed(preceded(txt_tag(txt), parse_forms))(input)?;
    let node = ParseNode::builder(kind, input, rest).children(args).build();
    Ok((rest, node))
}

////////////////////////////////////////////////////////////////////////////////
pub fn parse_arg(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::Arg, SyntaxErrorKind::*, TokenKind::*};
    let (rest, meta) = opt(parse_meta)(input)?;

    let (rest, matched) = wrap_err(InvalidArgument, |i| {
        parse_kind(i, [Identifier, FqnIdentifier], Arg)
    })(rest)?;

    let (rest,_opt) = opt(parse_type_annotation)(rest)?;
    Ok((rest, matched.change_meta(meta)))
}

pub fn parse_args(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::Args, SyntaxErrorKind::*, TokenKind::*};
    let (rest, matched) =
        parse_wrapped_many(OpenSquareBracket, CloseSquareBracket, "]", cut(parse_arg))(input)?;
    let node = ParseNode::builder(Args, input, rest).children(matched);
    Ok((rest, node.build()))
}

pub fn parse_lambda_body(input: Span) -> PResult<ParseNode> {
    let (rest, (args, forms)) = parse_bracketed(pair(parse_args, parse_forms))(input)?;

    let node = ParseNode::builder(AstNodeKind::LambdaBody, input, rest)
        .child(args)
        .children(forms)
        .build();
    Ok((rest, node))
}

pub fn parse_multi_lambda_body(input: Span) -> PResult<Vec<ParseNode>> {
    let (rest, lambdas) = many0(parse_lambda_body)(input)?;
    Ok((rest, lambdas))
}

pub fn parse_single_lambda_body(input: Span) -> PResult<Vec<ParseNode>> {
    let (rest, (args, forms)) = pair(parse_args, parse_forms)(input)?;

    let node = ParseNode::builder(AstNodeKind::LambdaBody, input, rest)
        .child(args)
        .children(forms)
        .build();
    Ok((rest, vec![node]))
}

pub fn parse_block(input: Span) -> PResult<ParseNode> {
    let (rest, forms) = parse_multi_lambda_body(input)?;

    let node = ParseNode::builder(AstNodeKind::Block, input, rest)
        .children(forms)
        .build();
    Ok((rest, node))
}

pub fn parse_lambda(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;

    let body = alt((parse_single_lambda_body, parse_multi_lambda_body));

    let (rest, lambdas) = preceded(
        pair(tag(OpenBracket), txt_tag("fn")),
        cut(succeeded(body, tag(CloseBracket))),
    )(input)?;

    let (rest,_ret_type) = opt(parse_type_annotation)(rest)?;

    let node = ParseNode::builder(AstNodeKind::Lambda, input, rest)
        .children(lambdas)
        .build();
    Ok((rest, node))
}

fn parse_list(input: Span) -> PResult<ParseNode> {
    use TokenKind::{CloseBracket, OpenBracket, Quote};
    let (rest, matched) = preceded(tag(Quote), parse_bracketed(many0(parse_atom)))(input)?;
    let node = ParseNode::builder(AstNodeKind::List, input, rest).children(matched);
    Ok((rest, node.build()))
}

fn parse_array(input: Span) -> PResult<ParseNode> {
    use AstNodeKind::Array;
    let (rest, matched) = parse_sq_bracketed(many0(parse_atom))(input)?;
    let node = ParseNode::builder(AstNodeKind::Array, input, rest).children(matched);
    Ok((rest, node.build()))
}

pub fn parse_pair(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    let (rest, (a, b)) = pair(parse_atom, parse_atom)(input)?;
    let node = ParseNode::builder(AstNodeKind::Pair, input, rest).children([a, b]);
    Ok((rest, node.into()))
}

pub fn parse_keyword_pair(input: Span) -> PResult<ParseNode> {
    use TokenKind::*;
    let (rest, (a, b)) = pair(parse_keyword, parse_atom)(input)?;
    let node = ParseNode::builder(AstNodeKind::KeyWordPair, input, rest).children([a, b]);
    Ok((rest, node.into()))
}

pub fn parse_keyword(input: Span) -> PResult<ParseNode> {
    use AstNodeKind::*;
    let (rest, _) = tag(TokenKind::KeyWord)(input)?;
    Ok((rest, ParseNode::builder(KeyWord, input, rest).into()))
}

pub fn parse_map(input: Span) -> PResult<ParseNode> {
    use AstNodeKind::*;
    let (rest, kids) = parse_braced(many0(parse_pair))(input)?;
    let node = ParseNode::builder(Map, input, rest).children(kids);
    Ok((rest, node.into()))
}
pub fn parse_single_meta(input: Span) -> PResult<ParseNode> {
    let (rest, _) = tag(TokenKind::Caret)(input)?;
    let (rest, tag) = parse_keyword(rest)?;
    let is_true = ParseNode::builder(AstNodeKind::Bool, input, rest).build();
    let pair = ParseNode::builder(AstNodeKind::KeyWordPair, input, rest)
        .children([tag, is_true])
        .build();
    let meta = ParseNode::builder(AstNodeKind::MetaData, input, rest)
        .child(pair)
        .build();

    Ok((rest, meta))
}

pub fn parse_meta_map(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::*, TokenKind::Caret};
    let (rest, _) = tag(Caret)(input)?;
    let (rest, kids) = parse_braced(many0(parse_keyword_pair))(rest)?;
    let node = ParseNode::builder(MetaData, input, rest).children(kids);
    Ok((rest, node.into()))
}

pub fn parse_meta(input: Span) -> PResult<ParseNode> {
    alt((parse_single_meta, parse_meta_map))(input)
}

pub fn parse_let(input: Span) -> PResult<ParseNode> {
    use {AstNodeKind::Let, TokenKind::*};

    let (rest, (args, forms)) = parse_bracketed(preceded(
        txt_tag("let"),
        cut(pair(parse_let_args, parse_forms)),
    ))(input)?;

    let node = ParseNode::builder(Let, input, rest)
        .child(args)
        .children(forms)
        .build();
    Ok((rest, node))
}

fn parse_atom(input: Span) -> PResult<ParseNode> {
    alt((
        parse_null,
        parse_keyword,
        parse_number,
        parse_string,
        parse_bool,
        parse_builtin,
        parse_symbol,
        parse_specials,
        parse_application,
        parse_array,
        parse_list,
        parse_quoted,
        parse_map,
    ))(input)
}

pub fn parse_program(input: Span) -> PResult<ParseNode> {
    let (rest, matched) = many0(parse_atom)(input)?;
    let x = ParseNode::builder(AstNodeKind::Program, input, rest).children(matched);
    Ok((rest, x.into()))
}
