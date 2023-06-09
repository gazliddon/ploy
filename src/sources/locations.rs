use super::prelude::*;

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum SourceOrigin {
    Text,
    File(FileId, PathBuf),
}

/// Describes a point in some text
#[derive(Clone, Debug, Copy)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

/// Describes a span in some text
#[derive(Clone, Debug, Copy)]
pub struct SourceSpan {
    pub location: Location,
    pub len: usize,
}

/// Describes a span of text in file
#[derive(Clone, Debug, Copy)]
pub struct FileSpan {
    pub id: FileId,
    pub span: SourceSpan,
}

