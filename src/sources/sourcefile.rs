use std::collections::HashMap;
use super::prelude::*;

#[derive(Clone, Debug)]
pub struct SourceFile {
    pub origin: SourceOrigin,
    pub text: String,
    lines: Lines,
}

impl SourceFile {
    pub fn new(text: String, origin: SourceOrigin) -> Self {
        Self {
            lines: Lines::new(&text),
            text,
            origin,
        }
    }

    pub fn get_location(&self, _offset: usize) -> Option<Location> {
        self.lines.get_location_from_offset(_offset)
    }

    pub fn get_line(&self, line: usize) -> Option<&str> {
        self.lines.get_line_range(line).map(|r| &self.text[r])
    }

    fn in_bounds(&self, r: &std::ops::Range<usize>) -> bool {
        let len = self.text.len();
        r.start < len && r.end < len
    }

    pub fn get_text(&self, r: std::ops::Range<usize>) -> Option<&str> {
        self.in_bounds(&r).then_some(&self.text[r])
    }

    pub fn get_file_span_from_offset(&self, offset: usize) -> Option<FileSpan> { 
        let loc = self.lines.get_location_from_offset(offset)?;
        Some(FileSpan::new(self.origin.clone(), loc.line, loc.col,0))
    }}



use std::fs::File;
use std::path::{Path, PathBuf};
use thin_vec::ThinVec;
use thiserror::Error;

struct SourceLoader {
    name_to_id: HashMap<PathBuf, FileId>,
    id_to_file: HashMap<FileId, SourceFile>,
    next_id: u64,
}

impl SourceLoader {
    pub fn new() -> Self {
        Self {
            name_to_id: Default::default(),
            id_to_file: Default::default(),
            next_id: 0,
        }
    }

    fn add_file<P: AsRef<Path>>(&mut self, p: P, text: String) -> FileId {
        let id = self.next_id;

        if self.get_source_file_from_name(&p).is_some() {
            panic!("Arleady exists");
        }

        let p = p.as_ref().to_path_buf();
        let source = SourceFile::new(text, SourceOrigin::File(id, p.clone()));
        self.name_to_id.insert(p.clone(), id);
        self.id_to_file.insert(id, source);

        self.next_id += 1;
        id
    }

    pub fn get_source_file_from_name<P: AsRef<Path>>(&self, p: P) -> Option<&SourceFile> {
        self.name_to_id
            .get(p.as_ref())
            .and_then(|id| self.get_source_file(*id))
    }

    pub fn get_source_file(&self, file_id: FileId) -> Option<&SourceFile> {
        self.id_to_file.get(&file_id)
    }

    pub fn get_source_file_id<P: AsRef<Path>>(&self, p: P) -> Option<FileId> {
        self.name_to_id.get(p.as_ref()).copied()
    }

    pub fn load_file<P: AsRef<Path>>(&mut self, p: P) -> Result<FileId, ()> {
        if let Some(SourceFile {
            origin: SourceOrigin::File(id, _),
            ..
        }) = self.get_source_file_from_name(p)
        {
            Ok(*id)
        } else {
            panic!()
        }
    }
}
