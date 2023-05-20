#![allow(dead_code)]
#![allow(unused_imports)]

mod cli;
mod opts;
mod frontend;
mod error;
use anyhow::Context;

fn main() -> anyhow::Result<()> {
    let opts = cli::parse_opts()?;
    println!("{opts:#?}");
    Ok(())
}
