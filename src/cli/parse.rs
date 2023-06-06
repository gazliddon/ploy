use crate::opts::{Action, Opts};
use clap::{Arg, ArgAction, Command};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use super::prelude::*;

////////////////////////////////////////////////////////////////////////////////
// App specific data
// Should be passed?
lazy_static::lazy_static! {
    static ref COMMANDS : Vec<CommandInfo> = vec![
        CommandInfo::new("build",Action::Build, "Build the project"),
        CommandInfo::new("check",Action::Check, "Check for errors"),
        CommandInfo::new("lsp",Action::Lsp, "Launch LSP server"),
    ];

    static ref COM_TO_COM_INFO : HashMap<&'static str,CommandInfo> = {
        COMMANDS.iter().map(|c| (c.id,*c)).collect()
    };

    static ref BUILD_INFO : BuildInfo = BuildInfo::new();
}

////////////////////////////////////////////////////////////////////////////////
struct Parser {
    default_project_file: String,
}

#[derive(Clone, Debug, Copy)]
pub struct CommandInfo {
    action: Action,
    id: &'static str,
    help_text: &'static str,
}

impl CommandInfo {
    pub fn new(id: &'static str, action: Action, help_text: &'static str) -> Self {
        Self {
            id,
            action,
            help_text,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

impl Parser {
    pub fn new(default_project_file: &str) -> Self {
        Self {
            default_project_file: default_project_file.to_owned(),
        }
    }

    fn make_config_file_arg(&self) -> clap::Arg {
        clap::Arg::new("config-file")
            .help("load config file")
            .index(1)
            .required(false)
    }

    fn load_opts(&self, m: &clap::ArgMatches, action: Action) -> Result<Opts, CliErrorKind> {
        let path: PathBuf = m
            .get_one::<String>("config-file")
            .unwrap_or(&self.default_project_file)
            .into();
        let opts = load_project_file(path)?;
        Ok(Opts { action, ..opts })
    }

    fn make_config_file_command(
        &self,
        command: &'static str,
        about: &'static str,
    ) -> clap::Command {
        clap::Command::new(command)
            .about(about)
            .arg(self.make_config_file_arg())
    }

    fn get_matches(&self) -> clap::ArgMatches {
        let mut com = Command::new("ploy")
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
            );

        for com_info in COM_TO_COM_INFO.values() {
            com = com.subcommand(self.make_config_file_command(com_info.id, com_info.help_text))
        }

        com.get_matches()
    }

    pub fn build(&self) -> Result<Opts, CliErrorKind> {
        use Action::*;

        let matches = self.get_matches();

        let Some((text,m)) = matches.subcommand() else {
            return Err(CliErrorKind::NoAction)
        };

        let Some(command) = COM_TO_COM_INFO.get(text) else {
            return Err(CliErrorKind::UnrecognisedCommand(text.to_string()))
        };

        self.load_opts(m, command.action)
    }
}

/// Find the ploy project file from a supplied path
/// If the path is a dir it will append the default_project_file
// fn find_project_file<P: AsRef<Path>>(
//     path: P,
//     default_project_file: &str,
// ) -> anyhow::Result<PathBuf> {
//     // check to see if the pass file is a dir?
//     let path = if path.as_ref().is_dir() {
//         path.as_ref().join(default_project_file)
//     } else {
//         path.as_ref().to_path_buf()
//     };

//     if path.is_dir() {
//         Err(anyhow::anyhow!("{} is a directory", path.to_string_lossy()))
//     } else if !path.exists() {
//         Err(anyhow::anyhow!(
//             "Can't find file {}",
//             path.to_string_lossy()
//         ))
//     } else {
//         Ok(path)
//     }
// }

pub fn parse_opts(default_project_file: &str) -> Result<Opts, CliErrorKind> {
    let p = Parser::new(default_project_file);
    p.build()
}
