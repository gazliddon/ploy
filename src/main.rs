#![allow(dead_code)]
#![allow(unused_imports)]

use ploy::{*, error::{PloyErrorKind, to_full_error}, frontend::{ ModuleJob, Module }};
use std::path::Path;
use anyhow::Context;

use frontend::FrontEndError;
use thiserror::Error;

use frontend::FrontEndCtx;
use opts::Opts;

fn main() -> anyhow::Result<()> {
    let opts = cli::parse_opts(opts::DEFAULT_PROJECT_FILE)?;

    let mut loader = sources::SourceLoader::new();

    let id = loader.load_file(&opts.project_file)?;
    let sf = loader.get_source_file(id).expect("source file");

    let job = ModuleJob::new(&opts,sf);
    let _module : Module = job.try_into()?;

    println!("Compiled fine");

    Ok(())
}

