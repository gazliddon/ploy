#![allow(unused)]

use std::path::Path;

use logos::{internal::CallbackResult, Source};
use ploy::{sources::SourceFile, *, error::to_full_error};

use frontend::*;
use parsers::*;
use unraveler::Parser;
use ploy::error::PloyErrorKind;

pub fn load_text<P: AsRef<Path>>(p : P) -> String {
    panic!()
}

pub fn as_ast<P>(text: &str, mut p: P) -> Result<Ast, PloyErrorKind>
where
    P: for<'a> Parser<Span<'a>, ParseNode, FrontEndError>,
{
    let source_file = SourceFile::new(text.to_owned(), sources::SourceOrigin::Text);

    let merr = |e : FrontEndError| -> PloyErrorKind {
        to_full_error(e,&source_file)
    };

    let tokes = tokenize(&source_file);
    let span = Span::from_slice(&tokes);
    let (rest, matched) = p.parse(span).map_err(merr)?;

    let ast = Ast::new(matched, tokes.as_slice(), source_file.clone());

    Ok(ast)
}

pub fn kids_kinds(node: AstNodeRef) -> Vec<AstNodeKind> {
    let kids_kinds: Vec<_> = node.children().map(|n| n.value().kind.clone()).collect();
    kids_kinds
}

pub fn check_node(n: AstNodeRef, k: AstNodeKind, kids: &[AstNodeKind]) {
    let kids_kinds: Vec<_> = n.children().map(|n| n.value().kind.clone()).collect();
    println!("Node: {:?} : {kids_kinds:?}", &n.value().kind);
    assert_eq!(n.value().kind, k);
    assert_eq!(kids, kids_kinds);
}

pub fn test_parser<P>(
    p: P,
    text: &str,
    kind: AstNodeKind,
    kids: &[AstNodeKind],
) -> Result<(), PloyErrorKind>
where
    P: for<'a> Parser<Span<'a>, ParseNode, FrontEndError>,
{
    let ast = as_ast(text, p)?;
    let n = ast.tree.root();
    check_node(n, kind, kids);
    Ok(())
}

pub fn test_parsers<P>(
    p: P,
    kind: AstNodeKind,
    data: &[(&str, Vec<AstNodeKind>)],
) -> Result<(), PloyErrorKind>
where
    P: for<'a> Parser<Span<'a>, ParseNode, FrontEndError> + Clone,
{

    for (text, kids_kinds) in data.iter() {
        test_parser(p.clone(), text, kind.clone(), kids_kinds)?;
    }

    Ok(())
}
