use super::prelude::*;

use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub enum SourceOrigin {
    Text,
    File(FileId, PathBuf),
}

/// Describes a point in some text
#[derive(Clone, Debug, Copy, PartialEq,Default)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}
impl Location {
    pub fn new(line: usize, col: usize) -> Self {
        Self {
            line,col
        }
    }
}

/// Describes a span in some text
#[derive(Clone, Debug, Copy)]
pub struct SourceSpan {
    pub location: Location,
    pub len: usize,
}

impl SourceSpan {
    pub fn new(line: usize, col: usize, len: usize) -> Self {
        Self {
            location: Location::new(line,col),
            len
        }
    }
}

/// Describes a span of text in file
#[derive(Clone, Debug)]
pub struct FileSpan {
    pub origin: SourceOrigin,
    pub span: SourceSpan,
}

impl FileSpan {
    pub fn new(origin: SourceOrigin, line: usize, col: usize, len: usize) -> Self {
        Self {
            origin,
            span: SourceSpan::new(line,col,len)
        }
    }
}


