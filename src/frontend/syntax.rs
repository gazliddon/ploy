/// Checks AST for Syntax errors
/// and other processing
use super::prelude::*;

use crate::error::to_full_error;
use crate::sources::SourceFile;
use crate::symbols::{ScopeId, SymbolTree};

use anyhow::Context;
use serde::Deserialize;
use std::sync::atomic::AtomicIsize;
use std::vec;
use std::{collections::HashMap, io::Cursor};
use thin_vec::ThinVec;
use thiserror::Error;
use unraveler::Item;

use itertools::Itertools;

#[derive(Debug, Error, Clone)]
pub enum SyntaxErrorKind {
    #[error("If needs 2 or more args")]
    NotEnoughIf,
    #[error("If must have 3 or less args")]
    TooManyIf,
    #[error("Not enough arguments")]
    NotEngoughArgs,
    #[error("Too many arguments")]
    TooManyArgs,
    #[error("Invalid argument")]
    InvalidArgument,
    #[error("Expected {0}")]
    Expected(String),

    #[error("Undefined symbol {0}")]
    UndefinedSymbol(String),
}

fn get_str<'a>(x: Token<'a>, txt: &'a str) -> &'a str {
    &txt[x.location.as_range()]
}

fn to_kinds(x: &[Token], txt: &str) -> Vec<(TokenKind, String)> {
    x.iter()
        .map(|x| (x.get_kind(), get_str(*x, txt).to_string()))
        .collect()
}

pub struct SyntaxCtx<'a> {
    syms: &'a mut SymbolTree,
    ast: &'a mut super::ast::Ast,
    text: &'a str,
}

fn num_of_children(n: AstNodeRef) -> usize {
    let mut ret = 0;

    for _ in n.children() {
        ret += 1
    }
    ret
}

fn check_args(args: &[AstNodeRef], min: usize, max: usize) -> Result<(), SyntaxErrorKind> {
    if args.len() < min {
        Err(SyntaxErrorKind::NotEngoughArgs)
    } else if args.len() > max {
        Err(SyntaxErrorKind::TooManyArgs)
    } else {
        Ok(())
    }
}

fn min_args(args: &[AstNodeRef], min: usize) -> Result<(), SyntaxErrorKind> {
    if args.len() < min {
        Err(SyntaxErrorKind::NotEngoughArgs)
    } else {
        Ok(())
    }
}

/// Get all of the ids of this node Recursively, depth first
fn get_rec_ids_inner(tree: &AstTree, id: AstNodeId, nodes: &mut Vec<AstNodeId>) {
    nodes.push(id);
    let kids = tree.get(id).unwrap().children().map(|n| n.id());
    for k in kids {
        get_rec_ids_inner(tree, k, nodes)
    }
}

fn get_rec_ids(tree: &AstTree, id: AstNodeId) -> Vec<AstNodeId> {
    let mut nodes = vec![];
    get_rec_ids_inner(tree, id, &mut nodes);
    nodes
}

fn get_rec_ids_with_scope_inner(
    tree: &AstTree,
    id: AstNodeId,
    mut current_scope: ScopeId,
    nodes: &mut Vec<(ScopeId, AstNodeId)>,
) -> ScopeId {
    let node = tree.get(id).unwrap();

    if let AstNodeKind::SetScope(x) = node.value().kind {
        current_scope = x;
    } else {
        nodes.push((current_scope, id));

        let kids = tree.get(id).unwrap().children().map(|n| n.id());

        for k in kids {
            current_scope = get_rec_ids_with_scope_inner(tree, k, current_scope, nodes);
        }
    }

    current_scope
}

fn get_rec_ids_with_scope(
    tree: &AstTree,
    id: AstNodeId,
    current_scope: ScopeId,
) -> Vec<(ScopeId, AstNodeId)> {
    let id = tree.get(id).unwrap().id();
    let mut nodes: Vec<_> = vec![];
    get_rec_ids_with_scope_inner(tree, id, current_scope, &mut nodes);
    nodes
}

impl Ast {
    pub fn process(
        &mut self,
        syms: &mut SymbolTree,
        source: &SourceFile,
    ) -> Result<(), FrontEndError> {
        self.add_scopes(syms, source)?;
        self.intern_symbol_assignments(syms, source)?;
        self.intern_refs(syms, source)?;
        self.create_values(syms, source)?;
        Ok(())
    }

    fn mk_error(&self, _node: AstNodeRef, _err: SyntaxErrorKind) -> SyntaxErrorKind {
        panic!()
    }

    /// If this node has it's own scope
    /// then
    /// previous node = create a unique new scope
    /// after node = return to the current scope
    fn set_scope_for_node(
        &mut self,
        syms: &mut SymbolTree,
        id: AstNodeId,
        current_scope: ScopeId,
    ) -> ScopeId {
        let mut n = self.tree.get_mut(id).unwrap();
        let v = n.value();

        if v.kind.creates_new_scope() {
            let new_scope_name = format!("scope_{}", syms.get_next_scope_id());
            let new_scope = syms.create_or_get_scope_for_parent(&new_scope_name, current_scope);
            let before = v.change_kind(AstNodeKind::SetScope(new_scope));
            let after = v.change_kind(AstNodeKind::SetScope(current_scope));
            n.insert_before(before);
            n.insert_after(after);
            new_scope
        } else {
            current_scope
        }
    }

    /// Recursively scopes nodes that need a unique scope
    fn scope_node_recursive(
        &mut self,
        syms: &mut SymbolTree,
        id: AstNodeId,
        mut current_scope: ScopeId,
    ) {
        current_scope = self.set_scope_for_node(syms, id, current_scope);

        let n = self.tree.get(id).unwrap();
        let k_ids: ThinVec<_> = n.children().map(|n| n.id()).collect();
        for id in k_ids {
            self.scope_node_recursive(syms, id, current_scope)
        }
    }

    fn change_node_kind(&mut self, id: AstNodeId, new_kind: AstNodeKind) {
        let mut sym = self.tree.get_mut(id).unwrap();
        sym.value().kind = new_kind
    }

    /// Add scope setting, unsetting for all forms that need it
    fn add_scopes(
        &mut self,
        syms: &mut SymbolTree,
        _source: &SourceFile,
    ) -> Result<(), FrontEndError> {
        let id = self.tree.root().id();
        let current_scope = syms.get_root_scope_id();
        self.scope_node_recursive(syms, id, current_scope);
        Ok(())
    }

    fn intern_refs(
        &mut self,
        syms: &mut SymbolTree,
        source: &SourceFile,
    ) -> Result<(), FrontEndError> {
        use symbols::SymbolResolutionBarrier;
        use SyntaxErrorKind::*;
        let id = self.tree.root().id();
        let root_scope = syms.get_root_scope_id();
        let nodes = get_rec_ids_with_scope(&self.tree, id, root_scope);

        for (scope, id) in nodes.into_iter() {
            let node = self.tree.get(id).unwrap();
            let v = &node.value();

            if v.kind == AstNodeKind::Symbol {
                let name = source.get_text(v.text_range.clone()).unwrap();
                let sym_id = syms
                    .resolve_label(name, scope, SymbolResolutionBarrier::Global)
                    .map_err(|_e| {
                        FrontEndError::new(UndefinedSymbol(name.to_owned()), v.text_range.clone())
                    })?;

                self.change_node_kind(id, AstNodeKind::InternedSymbol(sym_id))
            }
        }

        Ok(())
    }

    /// Change all symbol defs, lambdas and defines, to symbol ids
    fn intern_symbol_assignments(
        &mut self,
        syms: &mut SymbolTree,
        source: &SourceFile,
    ) -> Result<(), FrontEndError> {
        use AstNodeKind::*;
        let mut current_scope = syms.get_root_scope_id();
        let nodes = get_rec_ids(&self.tree, self.tree.root().id());

        for id in &nodes {
            let n = self.tree.get(*id).unwrap();
            let v = n.value();
            match v.kind {
                SetScope(id) => current_scope = id,
                Arg => {
                    let sym = self.tree.get(*id).unwrap();
                    let name = &source.text[sym.value().text_range.clone()];
                    let sym_id = syms
                        .create_symbol_in_scope(current_scope, name)
                        .expect("Symbol exists TODO: error properly");
                    self.change_node_kind(*id, AstNodeKind::InternedSymbol(sym_id))
                }
                _ => (),
            }
        }

        Ok(())
    }

    fn create_values(
        &mut self,
        _syms: &mut SymbolTree,
        _source: &SourceFile,
    ) -> Result<(), FrontEndError> {
        Ok(())
    }
}
