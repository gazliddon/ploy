use super::tokens::{ FragementLocation,ParseText, TokenKind,};
use logos::Logos;

pub type Token<'a> = super::tokens::Token<ParseText<'a>>;

fn to_tokens_kinds(program_txt: &str) -> Vec<(TokenKind, std::ops::Range<usize>)> {
    TokenKind::lexer(program_txt)
        .spanned()
        .map(|(tok_res, pos)| match tok_res {
            Ok(kind) => (kind, pos),
            Err(_) => (TokenKind::Error, pos),
        })
        .collect()
}

fn to_tokens<'a>(program_txt: &'a str) -> Vec<Token> {
    to_tokens_kinds(program_txt)
        .into_iter()
        .map(|(kind, r)| Token {
            kind,
            location: FragementLocation {
                loc: r.clone().into(),
                extra: ParseText::new(&program_txt[r]),
            },
        })
        .collect()
}

pub fn tokenize(program_txt: &str) -> Vec<Token> {
    let tokes = to_tokens(program_txt);
    println!("{tokes:#?}");
    tokes
}
