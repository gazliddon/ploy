#![allow(dead_code)]
#![allow(unused_imports)]

mod cli;
mod opts;
mod frontend;
mod error;
use anyhow::Context;
use toml::to_string;
use unraveler::Item;
use frontend::{tokens::TokenKind, ploytokens::Token};

use crate::frontend::{ploytokens::tokenize, ast::to_ast};
fn get_str<'a>(x : Token<'a>, txt: &'a str) -> &'a str {
    &txt[x.location.loc.as_range()]
}

fn to_kinds(x : &[Token], txt: &str) -> Vec<( TokenKind, String )> {
    x.iter().map(|x| (x.get_kind(),get_str(x.clone(), txt).to_string() )).collect()
}

fn main() -> anyhow::Result<()> {
    let opts = cli::parse_opts()?;

    let program_txt = std::fs::read_to_string(&opts.project_file).context("Can't load project file")?;
    let tokes = tokenize(&program_txt);
    let ast = to_ast(tokes.clone());

    println!("{:#?}", ast);

    for (k,frag) in to_kinds(&tokes, &program_txt) {
    println!("{k:?} {frag}");
    }

    Ok(())
}
