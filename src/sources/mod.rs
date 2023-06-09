mod sourcefile;
mod error;
mod searchpaths;
mod loader;
mod locations;
mod textlines;

pub (crate) mod prelude {
    use super::*;
    pub use error::*;
    pub use sourcefile::SourceFile;
    pub use searchpaths::{ PathSearcher, SearchPathsError };
    pub use loader::{ SourceLoader, FileId };
    pub use locations::*;
    pub (crate) use textlines::*;
}

pub use prelude::*;


