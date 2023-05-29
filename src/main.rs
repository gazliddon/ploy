#![allow(dead_code)]
#![allow(unused_imports)]

mod cli;
mod opts;
mod frontend;
mod error;
use anyhow::Context;

use crate::frontend::ploytokens::tokenize;

fn main() -> anyhow::Result<()> {
    let opts = cli::parse_opts()?;

    let program_txt = std::fs::read_to_string(&opts.project_file).context("Can't load project file")?;
    let tokes = tokenize(&program_txt);

    println!("OPTS\n{opts:#?}");
    println!("TOKENS\n{tokes:#?}");
    Ok(())
}
