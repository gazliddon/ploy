pub mod build_info;
/// Module to handle the command line interface
/// Can take args from the command line or a yaml file
/// Creates an Opts struct for the compiler to work on
pub mod command_line;
pub mod yaml;

use crate::{
    cli::build_info::BuildInfo,
    opts::{Action, Opts, DEFAULT_PROJECT_FILE},
};
use std::path::{Path, PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum CliErrorKind {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Find the ploy project file from a supplied path
/// If the path is a dir it will append the default ploy project
/// file - Ploy.toml
fn find_project_file<P: AsRef<Path>>(path: P) -> anyhow::Result<PathBuf> {
    // check to see if the pass file is a dir?
    let path = if path.as_ref().is_dir() {
        path.as_ref().join(DEFAULT_PROJECT_FILE)
    } else {
        path.as_ref().to_path_buf()
    };

    if path.is_dir() {
        Err(anyhow::anyhow!("{} is a directory", path.to_string_lossy()))
    } else if !path.exists() {
        Err(anyhow::anyhow!(
            "Can't find file {}",
            path.to_string_lossy()
        ))
    } else {
        Ok(path)
    }
}

fn make_config_file_arg() -> clap::Arg {
    clap::Arg::new("config-file")
        .help("load config file")
        .index(1)
        .required(false)
        .default_value(DEFAULT_PROJECT_FILE)
}

fn make_config_file_command(command: &'static str, about: &'static str) -> clap::Command {
    clap::Command::new(command)
        .about(about)
        .arg(make_config_file_arg())
}

lazy_static::lazy_static! {
    static ref BUILD_INFO : BuildInfo = BuildInfo::new();
}

fn get_matches() -> clap::ArgMatches {
    use clap::{Arg, ArgAction, Command};
    Command::new("ploy")
        .bin_name(BUILD_INFO.bin_name.as_str())
        .version(BUILD_INFO.version.as_str())
        .author(BUILD_INFO.authors.as_str())
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .help("Verbose mode")
                .short('v')
                .action(ArgAction::SetTrue)
                .global(true),
        )
        .subcommand(make_config_file_command("build", "Build the project"))
        .subcommand(make_config_file_command("check", "Check syntax"))
        .subcommand(make_config_file_command("lsp", "Launch LSP server"))
        .get_matches()
}

pub fn load_opts(m: &clap::ArgMatches, action: Action) -> Result<Opts, CliErrorKind> {
    let path: PathBuf = m.get_one::<String>("config-file").unwrap().into();
    let mut opts = yaml::load_project_file(path)?;
    opts.action = action;
    Ok(opts)
}

pub fn parse_opts() -> Result<Opts, CliErrorKind> {
    let matches = get_matches();
    match matches.subcommand() {
        Some(("build", m)) => load_opts(m, Action::Build),
        Some(("check", m)) => load_opts(m, Action::Check),
        Some(("lsp", m)) => load_opts(m, Action::Lsp),
        _ => panic!()
    }
}
