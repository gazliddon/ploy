#![allow(unused)]

mod common;
use common::*;
use ploy::{sources::SourceFile, *, error::PloyErrorKind};

use frontend::*;
use parsers::*;
use unraveler::Parser;

#[test]
fn test_lamda_body()-> Result<(), PloyErrorKind> { 
    use AstNodeKind::*;

    let test = vec![
        ("([a b] a b)", vec![Args, Symbol, Symbol]),
        ("([])", vec![Args]),
        ("([a b c] \"xxxxx\")", vec![Args, QuotedString]),

    ];

    test_parsers(parse_lambda_body, LambdaBody, &test)
}

#[test]
fn test_multi_lambda_body()-> Result<(), PloyErrorKind> { 
    use AstNodeKind::*;

    let text = "([a b] a) ([a b] a)";
    let ast = as_ast(text, parse_block)?;
    Ok(())

}

#[test]
fn test_define() -> Result<(), PloyErrorKind> {
    use AstNodeKind::*;

    let test = vec![
        ("(def a b)", vec![Arg, Symbol]),
        ("(define x (fn[a] 12))", vec![Arg, Lambda]),
        ("(define y ())", vec![Arg, Null]),
        (
            "(define  ^{:test b :spam \"hello\"} y  ())",
            vec![Arg, Null],
        ),
    ];

    test_parsers(parse_define, Define, &test)
}

#[test]
fn test_pair() -> Result<(), PloyErrorKind> {
    use AstNodeKind::*;
    let test = vec![(":keword a", vec![KeyWord, Symbol])];
    test_parsers(parse_pair, Pair, &test)
}
#[test]
fn test_meta() -> Result<(), PloyErrorKind> {
    use AstNodeKind::*;

    let test = vec![("^{}", vec![]), ("^{:name \"hello!\"}", vec![KeyWordPair])];

    test_parsers(parse_meta, MetaData, &test)
}

#[test]
fn test_keyword_pair() -> Result<(), PloyErrorKind> {
    use AstNodeKind::*;

    let test = vec![
        (":keword a", vec![KeyWord, Symbol]),
        (":whoops \"Hello there!\"", vec![KeyWord, QuotedString]),
    ];

    test_parsers(parse_keyword_pair, KeyWordPair, &test)
}

#[test]
fn test_map() -> Result<(), PloyErrorKind> {
    use AstNodeKind::*;

    let test = vec![("{}", vec![]), ("{ :keword a (hello) b}", vec![Pair, Pair])];

    test_parsers(parse_map, Map, &test)
}

#[test]
fn test_if() -> Result<(), PloyErrorKind> {
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
fn test_let() -> Result<(), PloyErrorKind> {
    use AstNodeKind::*;

    let test = vec![
        ("(let [a 10] (println a) true)", vec![LetArgs, Application, Bool]),
    ];

    test_parsers(parse_let, Let, &test)
}

#[test]
fn test_lambda() -> Result<(), PloyErrorKind> {
    use AstNodeKind::*;

    let test = vec![
        ("(fn ( [x a a] 1 ) ( [x a a] 1 )   )", vec![LambdaBody, LambdaBody]),
        ("(fn [x a a]  )", vec![LambdaBody]),
        ("(fn [a b x] (print b) )", vec![LambdaBody]),
        ( "(fn [a b x] (print b) (print c) '() :test)",
            vec![LambdaBody],
        ),
    ];

    test_parsers(parse_lambda, Lambda, &test)
}

#[test]
fn test_args() -> Result<(), PloyErrorKind> {
    use AstNodeKind::*;

    let test = vec![
        ("[]", vec![]),
        ("[a b c]", vec![Arg, Arg, Arg]),
        ("[z]", vec![Arg]),
    ];

    test_parsers(parse_args, Args, &test)
}
