#![allow(dead_code)]
#![allow(unused_imports)]

mod cli;
mod error;
mod frontend;
mod opts;
mod symbols;
mod value;
mod ir;

use anyhow::Context;
use thiserror::Error;
use toml::to_string;
use unraveler::Item;

fn main() -> anyhow::Result<()> {
    use frontend::*;

    let opts = cli::parse_opts(opts::DEFAULT_PROJECT_FILE)?;

    let mut syms = crate::symbols::SymbolTree::new();

    let program_txt =
        std::fs::read_to_string(opts.project_file).context("Can't load project file")?;
    let tokes = tokenize(&program_txt);

    let mut ast = to_ast(&tokes)?;
    ast.process(&mut syms, &program_txt)?;

    println!("{:#?}", ast);
    Ok(())
}
