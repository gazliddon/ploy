use std::path::Path;
use crate::opts::Opts;
use serde::Deserialize;

use super::CliErrorKind;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
#[serde(rename_all = "kebab-case")]
pub struct ProjectFile {
    pub opts: Option<Opts>,
}

pub fn load_project_file<P: AsRef<Path>>(path : P) -> Result<Opts,CliErrorKind> {
    use anyhow::Context;
    let path = path.as_ref();
    let f = std::fs::read_to_string(path).with_context(|| format!("Can't load configuration file: {}", path.to_string_lossy()))?;
    let config : ProjectFile = toml::from_str(&f).context("Yaml deserializing error")?;
    let opts = config.opts.unwrap_or_default();
    Ok(opts)
}
