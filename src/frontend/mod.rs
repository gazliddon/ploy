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
mod semantics;

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
    pub use super::span::{ Span,get_text_range };
    pub use super::tokens::{ParseText, TokenKind};
    pub use super::types::*;
}

pub use prelude::*;

use crate::error::to_full_error;
use crate::error::PloyErrorKind;
use crate::opts::Opts;
use crate::sources::SourceOrigin;
use crate::sources::{SourceFile, SourceLoader};
use crate::symbols::ScopeId;
use crate::symbols::SymbolTree;
use std::collections::HashMap;
use std::path::Path;

use self::syntax::AstLowerer;

pub struct FrontEndCtx {
    syms: SymbolTree,
    opts: Opts,
    ast: Ast,
}

#[derive(Clone, Debug)]
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
    pub id_to_scope: HashMap<AstNodeId, ScopeId>,
}

impl Module {

    pub fn get_scope_for_node(&self, id: AstNodeId) -> Option<ScopeId> {
        self.id_to_scope.get(&id).cloned()
    }

}

impl TryFrom<ModuleJob> for Module {
    type Error = PloyErrorKind;

    fn try_from(module_job: ModuleJob) -> Result<Self, Self::Error> {
        let mut syms = SymbolTree::new();

        let tokes = tokenize(&module_job.source);
        let mut ast =
            to_ast(&tokes, module_job.source.clone()).map_err(|e| to_full_error(e, &module_job.source))?;

        let mut ast_lowerer = AstLowerer {
            syms: &mut syms,
            ast: &mut ast,
            id_to_scope: HashMap::new(),
        };

        ast_lowerer
            .lower()
            .map_err(|e| to_full_error(e, &module_job.source))?;

        let ret = Self {
            id_to_scope: ast_lowerer.id_to_scope,
            syms,
            ast,
            from: module_job.clone(),
        };

        Ok(ret)
    }
}
