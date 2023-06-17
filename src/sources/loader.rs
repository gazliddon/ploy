use anyhow::Context;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use thin_vec::ThinVec;

use super::prelude::*;

pub type FileId = u64;

pub struct SourceLoader {
    name_to_id: HashMap<PathBuf, FileId>,
    id_to_file: HashMap<FileId, SourceFile>,
    next_id: u64,
    searcher: PathSearcher,
}

impl SourceLoader {
    pub fn new() -> Self {
        let mut ret = Self {
            name_to_id: Default::default(),
            id_to_file: Default::default(),
            next_id: 0,
            searcher: PathSearcher::new(),
        };

        ret.searcher.add_path(".").unwrap();
        ret
    }

    pub fn resolve_file_path<P: AsRef<Path>>(&self, p: P) -> Result<PathBuf, SourcesError> {
        let ret  = self.searcher.search(&p)?;
        Ok(ret)
    }

    fn add_file<P: AsRef<Path>>(&mut self, p: P, text: String) -> Result<FileId, SourcesError> {
        let id = self.next_id;

        if self.get_source_file_from_name(&p).is_ok() {
            Err(SourcesError::SourceIsAlreadyInDatabase)
        } else {
            let p = p.as_ref().to_path_buf();
            let source = SourceFile::new(text, SourceOrigin::File(id, p.clone()));
            self.name_to_id.insert(p.clone(), id);
            self.id_to_file.insert(id, source);

            self.next_id += 1;
            Ok(id)
        }
    }

    pub fn get_source_file_from_name<P: AsRef<Path>>(&self, p: P) -> Result<&SourceFile, SourcesError> {
        self.name_to_id
            .get(p.as_ref()).ok_or(SourcesError::NoSourceFile)
            .and_then(|id| self.get_source_file(*id))
    }

    pub fn get_source_file(&self, file_id: FileId) -> Result<&SourceFile, SourcesError> {
        self.id_to_file.get(&file_id).ok_or(SourcesError::IllegalSourceId)
    }

    pub fn get_source_file_id<P: AsRef<Path>>(&self, p: P) -> Option<FileId> {
        self.name_to_id.get(p.as_ref()).copied()
    }

    pub fn load_file<P: AsRef<Path>>(&mut self, p: P) -> Result<FileId, SourcesError> {
        let p = self.searcher.search(&p)?;

        if let Ok(SourceFile {
            origin: SourceOrigin::File(id, _),
            ..
        }) = self.get_source_file_from_name(&p)
        {
            Ok(*id)
        } else {
            let program_txt = std::fs::read_to_string(&p).unwrap();
            self.add_file(&p, program_txt)
        }
    }
}
