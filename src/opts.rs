use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Copy)]
pub enum Action {
    Build,
    Check,
    Lsp,
}

#[derive(Default,Debug, Clone, Deserialize, Copy)]
pub enum Verbosity {
    Silent,
    #[default]
    Normal,
    Info,
    Debug,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[serde(default)]
pub struct Opts {
    pub project_file: PathBuf,
    pub action: Action,
    pub verbosity: Verbosity,
}

pub const DEFAULT_PROJECT_FILE : &str = "Ploy.toml";

impl Default for Opts {
    fn default() -> Self {
        Self {
            project_file: DEFAULT_PROJECT_FILE.to_owned().into(),
            action: Action::Check,
            verbosity: Default::default(),
        }
    }
}
