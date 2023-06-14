#![allow(dead_code)]
#![allow(unused_imports)]

use ploy::{*, error::{PloyErrorKind, to_full_error}};
use std::path::Path;
use anyhow::Context;

use frontend::FrontEndError;
use thiserror::Error;

use frontend::Ast;
use opts::Opts;

fn file_to_ast<P: AsRef<Path>>(_opts: Opts, p : P) -> Result<Ast,PloyErrorKind> {
    use frontend::*;
    let mut syms = crate::symbols::SymbolTree::new();
    let mut loader = sources::SourceLoader::new();
    let source_id = loader.load_file(p).expect("A source file");
    let sf = loader.get_source_file(source_id).expect("My source file");
    let tokes = tokenize(sf);
    let mut ast = to_ast(&tokes, sf.clone())?;
    ast.process(&mut syms, sf).map_err(|e| to_full_error(e, &sf))?;
    Ok(ast)
}

fn main() -> anyhow::Result<()> {
    let opts = cli::parse_opts(opts::DEFAULT_PROJECT_FILE)?;

    let _ast = file_to_ast(opts.clone(), &opts.project_file)?;

    println!("Compiled fine");

    Ok(())
}

