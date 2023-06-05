/// Checks AST for Syntax errors
/// and other processing
use super::prelude::*;

use crate::symbols::{ScopeId, SymbolTree};
use anyhow::Context;
use serde::Deserialize;
use std::{collections::HashMap, io::Cursor};
use thin_vec::ThinVec;
use thiserror::Error;
use unraveler::Item;

#[derive(Debug, Error)]
pub enum SyntaxErrorKind {
    #[error("If needs 2 or more args")]
    NotEnoughIf,
    #[error("If must have 3 or less args")]
    TooManyIf,
    #[error("Not enough arguments")]
    NotEngoughArgs,
    #[error("Too many arguments")]
    TooManyArgs,
}

fn get_str<'a>(x: Token<'a>, txt: &'a str) -> &'a str {
    &txt[x.location.loc.as_range()]
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

fn get_rec_ids(tree: &AstTree, id: AstNodeId, nodes: &mut Vec<AstNodeId>) {
    nodes.push(id);
    let kids = tree.get(id).unwrap().children().map(|n| n.id());
    for k in kids {
        get_rec_ids(tree, k, nodes)
    }
}

impl Ast {
    pub fn process(&mut self, syms: &mut SymbolTree, source: &str) -> Result<(), FrontEndError> {
        self.add_scopes(syms, source)?;
        self.intern_symbols(syms, source)?;
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

    /// Add scope setting, unsetting for all forms that need it
    fn add_scopes(&mut self, syms: &mut SymbolTree, _source: &str) -> Result<(), FrontEndError> {
        let id = self.tree.root().id();
        let current_scope = syms.get_root_scope_id();
        self.scope_node_recursive(syms, id, current_scope);
        Ok(())
    }

    /// Change all symbols to interned symbols
    fn intern_symbols(&mut self, syms: &mut SymbolTree, source: &str) -> Result<(), FrontEndError> {
        use AstNodeKind::*;

        let mut current_scope = syms.get_root_scope_id();

        let mut nodes = vec![];
        get_rec_ids(&self.tree, self.tree.root().id(), &mut nodes);

        for id in nodes {
            let n = self.tree.get(id).unwrap();
            let v = n.value();

            match v.kind {
                SetScope(id) => current_scope = id,

                Let => {
                    println!("Let");
                    let args = n.first_child().unwrap();
                    for sym in args.children() {
                        let name = &source[sym.value().text_range.clone()];
                        println!("{current_scope} {name}");
                    }
                }

                Define => {
                    println!("Define");
                    let sym = n.first_child().unwrap();
                    let name = &source[sym.value().text_range.clone()];
                    println!("{current_scope} {name}");
                }

                Lambda => {
                    println!("Lambda");
                    let args = n.first_child().unwrap();
                    for sym in args.children() {
                        let name = &source[sym.value().text_range.clone()];
                        println!("{current_scope} {name}");
                    }
                }

                _ => (),
            }
        }

        Ok(())
    }

    fn create_values(
        &mut self,
        _syms: &mut SymbolTree,
        _source: &str,
    ) -> Result<(), FrontEndError> {
        Ok(())
    }
}
