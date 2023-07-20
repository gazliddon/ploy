// handles semantic analysis
// So...
// function arrity
// types

use super::prelude::*;
use thiserror::Error;

pub struct SemanticAnalyzer<'a> {
    module : &'a mut Module,
}

#[derive(Debug, Error, Clone)]
pub enum SemanticErrorKind {
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(module: &'a mut Module) -> Self {
        Self {
            module
        }
    }

    pub fn analyze(&mut self) -> Result<(),FrontEndError> {
        panic!()
    }
}
