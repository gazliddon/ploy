/// Module to handle the command line interface
/// Can take args from the command line or a yaml file
/// Creates an Opts struct for the compiler to work on
mod build_info;
mod command_line;
mod yaml;
mod error;
mod parse;

pub mod prelude {
    pub use super::error::*;
    pub use super::parse::parse_opts;
    pub (crate) use super::build_info::*;
    pub (crate) use super::yaml::*;
}

pub use prelude::*;


