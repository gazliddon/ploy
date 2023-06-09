mod sourcefile;
mod error;
mod searchpaths;
mod loader;

pub (crate) mod prelude {
    use super::*;
    pub use error::*;
    pub use sourcefile::{SourceFile, SourceOrigin, FileId };
    pub use searchpaths::{ PathSearcher, SearchPathsError };
    pub use loader::SourceLoader;
}

pub use prelude::*;


