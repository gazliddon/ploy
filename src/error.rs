use crate::cli::CliErrorKind;
use crate::frontend::FrontEndError;
use crate::sources::{SourcesError, FileSpan, SourceFile};

#[derive(thiserror::Error)]

pub enum PloyErrorKind {
    #[error(transparent)]
    Cli(#[from] CliErrorKind),

    #[error("{0}")]
    FrontEnd(#[from] FrontEndError),

    #[error(transparent)] 
    SourceError(#[from] SourcesError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl std::fmt::Debug for PloyErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(e) => write!(f,"{}", e.to_string()),
            _ => write!(f,"{}",self)
        }
    }
}


pub fn to_full_error(e: FrontEndError, source_file : &SourceFile) -> PloyErrorKind {
    let loc = source_file.get_file_span_from_offset(e.pos.start).unwrap();
    let line = loc.span.location.line;
    let col = loc.span.location.col;
    let line_text = source_file.get_line(line).unwrap();
    let spaces = " ".repeat(col);
    let text = format!("{e}\nFile: {:?}\nLine: {} Col: {}\n\n{line_text}\n{}^", loc.origin, line +1, col+1, spaces);
    let err = anyhow::anyhow!(text);
    PloyErrorKind::Other(err)
}



