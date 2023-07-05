/// Compiler front end
/// tokenising and parsing into an AST
mod ast;
mod error;
mod parsenode;
pub mod parsers;
mod ploytokens;
mod span;
mod syntax;
mod tokens;
mod types;

mod prelude {
    pub use super::{
        ast::{to_ast, Ast, AstNode, AstNodeKind},
        ast::{AstNodeId, AstNodeMut, AstNodeRef},
        error::{FrontEndError, FrontEndErrorKind, PResult},
        ploytokens::tokenize,
    };

    pub use super::ast::AstTree;
    pub use super::parsenode::ParseNode;
    pub use super::ploytokens::Token;
    pub use super::span::Span;
    pub use super::tokens::{ParseText, TokenKind};
    pub use super::types::*;
}

pub use prelude::*;

use crate::error::to_full_error;
use crate::error::PloyErrorKind;
use crate::opts::Opts;
use crate::sources::SourceOrigin;
use crate::sources::{SourceFile, SourceLoader};
use crate::symbols::SymbolTree;
use std::path::Path;

use self::syntax::AstLowerer;

pub struct FrontEndCtx {
    syms: SymbolTree,
    opts: Opts,
    ast: Ast,
}

#[derive(Clone,Debug)]
pub struct ModuleJob {
    source: SourceFile,
    opts: Opts,
}

impl ModuleJob {
    pub fn new(opts: &Opts, source: &SourceFile) -> Self {
        Self {
            opts: opts.clone(),
            source: source.clone(),
        }
    }
}

pub struct Module {
    pub syms: SymbolTree,
    pub ast: Ast,
    pub from: ModuleJob,
}

impl TryFrom<ModuleJob> for Module {
    type Error = PloyErrorKind;

    fn try_from(value: ModuleJob) -> Result<Self, Self::Error> {

        let from = value.clone();
        let mut syms = SymbolTree::new();

        let tokes = tokenize(&value.source);
        let mut ast = to_ast(&tokes, value.source.clone())?;

        let mut ast_lowerer = AstLowerer {
            syms: &mut syms,
            ast: &mut ast,
        };

        ast_lowerer.lower()
            .map_err(|e| to_full_error(e, &value.source))?;

        let ret = Self { syms, ast, from };

        Ok(ret)
    }
}

