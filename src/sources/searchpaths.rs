use std::path::{Path, PathBuf};
use thin_vec::ThinVec;
use thiserror::Error;


#[derive(Clone, Error, Debug)]
pub enum SearchPathsError {
    #[error("Can't find file")]
    NotInPath(ThinVec<PathBuf>),
    #[error("Can't get working directory")]
    NoWorkingDir,
    #[error("Tried to remove a search path item not in search path")]
    NotInSearchPath,
    #[error("Tried to add a search path item already in search path")]
    AlreadyInSearchPath,
}

pub struct PathSearcher {
    paths: Vec<PathBuf>,
}

impl PathSearcher {
    pub fn new()-> Self {
        Self {
            paths: Default::default(),
        }
    }

    pub fn add_path<P: AsRef<Path>>(&mut self, p: P) -> Result<(), SearchPathsError> {
        let p = p.as_ref().to_path_buf();
        if self.paths.contains(&p) {
            Err(SearchPathsError::AlreadyInSearchPath)
        } else {
            self.paths.push(p);
            Ok(())
        }
    }

    pub fn remove_path<P: AsRef<Path>>(&mut self, p: P) -> Result<(), SearchPathsError> {
        let p = p.as_ref().to_path_buf();

        if let Some(pos) = self.paths.iter().position(|x| x == &p) {
            self.paths.remove(pos);
            Ok(())
        } else {
            Err(SearchPathsError::NotInSearchPath)
        }
    }

    /// Search for a file from the current directory
    pub fn search<P: AsRef<Path>>(&self, file: P) -> Result<PathBuf, SearchPathsError> {
        if let Ok(from_dir) = std::env::current_dir() {
            self.search_from(from_dir, file)
        } else {
            Err(SearchPathsError::NoWorkingDir)
        }
    }

    /// Search for a file from a directory
    /// returns the found file
    pub fn search_from<P1: AsRef<Path>, P2: AsRef<Path>>(
        &self,
        from_dir: P1,
        file: P2,
    ) -> Result<PathBuf, SearchPathsError> {
        let from_dir = from_dir.as_ref().to_path_buf();

        let mut tried = ThinVec::new();

        for f in &self.paths {
            let mut f = f.to_path_buf();

            if f.is_relative() {
                f = from_dir.join(f)
            };

            let f = f.join(&file);

            if f.is_file() && f.exists() {
                return Ok(f);
            }
            tried.push(f)
        }

        Err(SearchPathsError::NotInPath(tried))
    }
}
