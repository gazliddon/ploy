use crate::error::to_full_error;
use crate::error::PloyErrorKind;
use crate::opts::Opts;
use std::collections::HashMap;
use std::path::Path;
use super::prelude::*;
use crate::sources::SourceOrigin;
use crate::sources::{SourceFile, SourceLoader};
use crate::symbols::ScopeId;
use crate::symbols::SymbolTree;
use super::syntax::AstLowerer;

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
