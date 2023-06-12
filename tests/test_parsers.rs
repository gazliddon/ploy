#![allow(unused)]

mod common;
use common::*;
use logos::{internal::CallbackResult, Source};
use ploy::{sources::SourceFile, *};

use frontend::*;
use parsers::*;

struct Ctx {
    source_file: SourceFile,
}

impl Ctx {
    fn new(txt: &str) -> Self {
        Self {
            source_file: SourceFile::new(txt.to_owned(), sources::SourceOrigin::Text),
        }
    }

    fn get_tokes<'a>(&'a self) -> Vec<Token<'a>> {
        let tokes = tokenize(&self.source_file);
        tokes
    }
}
use unraveler::Parser;

fn as_ast<P>(text: &str, mut p: P) -> Result<Ast, FrontEndError>
where
    P: for<'a> Parser<Span<'a>, ParseNode, FrontEndError>,
{
    let source_file = SourceFile::new(text.to_owned(), sources::SourceOrigin::Text);

    let tokes = tokenize(&source_file);
    let span = Span::from_slice(&tokes);
    let (rest, matched) = p.parse(span)?;

    let ast = Ast::new(matched, tokes.as_slice(), source_file.clone());

    Ok(ast)
}

fn kids_kinds(node: AstNodeRef) -> Vec<AstNodeKind> {
    let kids_kinds: Vec<_> = node.children().map(|n| n.value().kind.clone()).collect();
    kids_kinds
}

fn check_node(n: AstNodeRef, k: AstNodeKind, kids: &[AstNodeKind]) {
    let kids_kinds: Vec<_> = n.children().map(|n| n.value().kind.clone()).collect();
    println!("Node: {:?} : {kids_kinds:?}", &n.value().kind);
    assert_eq!(n.value().kind, k);
    assert_eq!(kids, kids_kinds);
}

fn test_parser<P>(
    p: P,
    text: &str,
    kind: AstNodeKind,
    kids: &[AstNodeKind],
) -> Result<(), FrontEndError>
where
    P: for<'a> Parser<Span<'a>, ParseNode, FrontEndError>,
{
    let ast = as_ast(text, p)?;
    let n = ast.tree.root();
    check_node(n, kind, kids);
    Ok(())
}

fn test_parsers<P>(
    p: P,
    kind: AstNodeKind,
    data: &[(&str, Vec<AstNodeKind>)],
) -> Result<(), FrontEndError>
where
    P: for<'a> Parser<Span<'a>, ParseNode, FrontEndError> + Clone,
{
    for (text, kids_kinds) in data.iter() {
        test_parser(p.clone(), text, kind.clone(), kids_kinds)?;
    }
    Ok(())
}

////////////////////////////////////////////////////////////////////////////////
#[test]
fn test_define() -> Result<(), FrontEndError> {
    use AstNodeKind::*;

    let test = vec![
        ("(def a b)", vec![Symbol, Symbol]),
        ("(define x (fn[a] 12))", vec![Symbol, Lambda]),
        ("(define y ())", vec![Symbol, Null]),
        (
            "(define  y ^{:test b :spam \"hello\"} ())",
            vec![Symbol, Null],
        ),
    ];

    test_parsers(parse_define, Define, &test)
}

#[test]
fn test_pair() -> Result<(), FrontEndError> {
    use AstNodeKind::*;

    let test = vec![(":keword a", vec![KeyWord, Symbol])];

    test_parsers(parse_pair, Pair, &test)
}
#[test]
fn test_meta() -> Result<(), FrontEndError> {
    use AstNodeKind::*;

    let test = vec![("^{}", vec![]), ("^{:name \"hello!\"}", vec![KeyWordPair])];

    test_parsers(parse_meta, MetaData, &test)
}

#[test]
fn test_keyword_pair() -> Result<(), FrontEndError> {
    use AstNodeKind::*;

    let test = vec![
        (":keword a", vec![KeyWord, Symbol]),
        (":whoops \"Hello there!\"", vec![KeyWord, QuotedString]),
    ];

    test_parsers(parse_keyword_pair, KeyWordPair, &test)
}

#[test]
fn test_map() -> Result<(), FrontEndError> {
    use AstNodeKind::*;

    let test = vec![("{}", vec![]), ("{ :keword a (hello) b}", vec![Pair, Pair])];

    test_parsers(parse_map, Map, &test)
}

#[test]
fn test_if() -> Result<(), FrontEndError> {
    use AstNodeKind::*;

    let test = vec![
        ("(if a b)", vec![Symbol, Symbol]),
        ("(if :a b :a)", vec![KeyWord, Symbol, KeyWord]),
        ("(if a b 12)", vec![Symbol, Symbol, Number]),
        ("(if a b (x a))", vec![Symbol, Symbol, Application]),
        ("(if a b ())", vec![Symbol, Symbol, Null]),
    ];

    test_parsers(parse_if, If, &test)
}

#[test]
fn test_let() -> Result<(), FrontEndError> {
    use AstNodeKind::*;

    let test = vec![
        ("(let [a 10] (println a) true)", vec![LetArgs, Application, Bool]),
    ];

    test_parsers(parse_let, Let, &test)
}

#[test]
fn test_lambda() -> Result<(), FrontEndError> {
    use AstNodeKind::*;

    let test = vec![
        ("(fn []  )", vec![Args]),
        ("(fn [a b x] (print b) )", vec![Args, Application]),
        (
            "(fn [a b x] (print b) (print c) '() :test)",
            vec![Args, Application, Application, List, KeyWord],
        ),
    ];

    test_parsers(parse_lambda, Lambda, &test)
}

#[test]
fn test_args() -> Result<(), FrontEndError> {
    use AstNodeKind::*;

    let test = vec![
        ("[]", vec![]),
        ("[a b c]", vec![Arg, Arg, Arg]),
        ("[z]", vec![Arg]),
    ];

    test_parsers(parse_args, Args, &test)
}
