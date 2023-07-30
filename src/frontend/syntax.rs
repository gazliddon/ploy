use super::ast::{ApplicationData, IfData};
/// Checks AST for Syntax errors
/// Does some AST lowering
/// and other processing
use super::prelude::*;

use crate::error::to_full_error;
use crate::sources::SourceFile;
use crate::symbols::{ScopeId, SymbolTree};
use crate::value;

use anyhow::Context;
use serde::Deserialize;
use std::sync::atomic::AtomicIsize;
use std::vec;
use std::{collections::HashMap, io::Cursor};
use thin_vec::ThinVec;
use thiserror::Error;
use unraveler::Item;

use itertools::Itertools;
use symbols::SymbolResolutionBarrier;

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
    #[error("Unexpected input")]
    Unexpected,

    #[error("This isn't something you can call")]
    IllegalApplication,
}

fn get_str<'a>(x: Token<'a>, txt: &'a str) -> &'a str {
    &txt[x.location.as_range()]
}

fn to_kinds(x: &[Token], txt: &str) -> Vec<(TokenKind, String)> {
    x.iter()
        .map(|x| (x.get_kind(), get_str(*x, txt).to_string()))
        .collect()
}

pub struct AstLowerer<'a> {
    pub syms: &'a mut SymbolTree,
    pub ast: &'a mut super::ast::Ast,
    pub id_to_scope: HashMap<AstNodeId, ScopeId>,
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

impl<'a> AstLowerer<'a> {
    pub fn lower(&mut self) -> Result<(), FrontEndError> {
        self.add_scopes()?;
        self.intern_symbol_assignments()?;
        self.intern_refs()?;
        self.create_values()?;
        self.process_special_forms()?;

        self.make_node_to_scope_table();

        Ok(())
    }

    fn make_node_to_scope_table(&mut self) {
        let nodes = self
            .get_node_values_with_scope(self.ast.tree.root().id(), self.syms.get_root_scope_id());

        for (id, _, scope) in nodes.into_iter() {
            self.id_to_scope.insert(id, scope);
        }
    }

    fn mk_error(&self, _node: AstNodeRef, _err: SyntaxErrorKind) -> SyntaxErrorKind {
        panic!()
    }

    /// If this node has it's own scope
    /// then
    /// previous node = create a unique new scope
    /// after node = return to the current scope
    fn set_scope_for_node(&mut self, id: AstNodeId, current_scope: ScopeId) -> ScopeId {
        let mut n = self.ast.tree.get_mut(id).unwrap();
        let v = n.value();

        if v.kind.creates_new_scope() {
            let new_scope_name = format!("scope_{}", self.syms.get_next_scope_id());
            let new_scope = self
                .syms
                .create_or_get_scope_for_parent(&new_scope_name, current_scope);
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
    fn scope_node_recursive(&mut self, id: AstNodeId, mut current_scope: ScopeId) {
        current_scope = self.set_scope_for_node(id, current_scope);

        let n = self.ast.tree.get(id).unwrap();
        let k_ids: ThinVec<_> = n.children().map(|n| n.id()).collect();
        for id in k_ids {
            self.scope_node_recursive(id, current_scope)
        }
    }

    /// Add scope setting, unsetting for all forms that need it
    fn add_scopes(&mut self) -> Result<(), FrontEndError> {
        let id = self.ast.tree.root().id();
        let current_scope = self.syms.get_root_scope_id();
        self.scope_node_recursive(id, current_scope);
        Ok(())
    }

    fn intern_refs(&mut self) -> Result<(), FrontEndError> {
        let nodes = self
            .get_node_values_with_scope(self.ast.tree.root().id(), self.syms.get_root_scope_id());

        for (id, v, current_scope) in nodes.into_iter() {
            use {SymbolResolutionBarrier::Global, SyntaxErrorKind::*};
            if v.kind == AstNodeKind::ToProcess(ToProcessKind::Symbol) {
                let r = &v.text_range;
                let name = self.get_source_text(r);
                let sym_id = self
                    .syms
                    .resolve_label(name, current_scope, Global)
                    .map_err(|_e| FrontEndError::new(UndefinedSymbol(name.to_owned()), r))?;

                self.change_node_kind(id, AstNodeKind::Symbol(sym_id))
            }
        }

        Ok(())
    }

    fn get_node_values_with_scope(
        &self,
        id: AstNodeId,
        current_scope: ScopeId,
    ) -> Vec<(AstNodeId, AstNode, ScopeId)> {
        let mut current_scope = current_scope;
        let mut ret = vec![];
        for id in self.ast.get_rec_ids(id).into_iter() {
            let n = self.ast.tree.get(id).unwrap();
            let v = n.value();
            match v.kind {
                AstNodeKind::SetScope(id) => current_scope = id,
                _ => ret.push((n.id(), v.clone(), current_scope)),
            }
        }
        ret
    }
    /// Lower defines to include the symbol id
    fn lower_defines(
        &mut self,
        syms: &mut SymbolTree,
        _: &SourceFile,
    ) -> Result<(), FrontEndError> {
        let nodes =
            self.get_node_values_with_scope(self.ast.tree.root().id(), syms.get_root_scope_id());

        for (id, value, _) in nodes.into_iter() {
            if value.kind == AstNodeKind::Define {
                let n = self.ast.tree.get(id).unwrap();
                let sym = n.first_child().unwrap();
                if let AstNodeKind::Symbol(sym_id) = sym.value().kind {
                    self.ast.tree.get_mut(sym.id()).unwrap().detach();
                    self.change_node_kind(id, AstNodeKind::AssignSymbol(sym_id));
                } else {
                    panic!("Whoops!");
                }
            }
        }

        Ok(())
    }

    fn detach_node(&mut self, id: AstNodeId) {
        self.ast.tree.get_mut(id).unwrap().detach();
    }

    /// Change all symbol defs, lambdas and defines, to symbol ids
    fn intern_symbol_assignments(&mut self) -> Result<(), FrontEndError> {
        let nodes = self
            .get_node_values_with_scope(self.ast.tree.root().id(), self.syms.get_root_scope_id());

        for (id, value, current_scope) in nodes.into_iter() {
            if value.kind == AstNodeKind::Arg {
                let name = self.get_source_text(&value.text_range).to_owned();
                let sym_id = self
                    .syms
                    .create_symbol_in_scope(current_scope, &name)
                    .expect("Symbol exists TODO: error properly");
                self.change_node_kind(id, AstNodeKind::Symbol(sym_id))
            }
        }

        Ok(())
    }

    /// Go over the special forms and wrap up the data nicely fro codegen
    fn process_special_forms(&mut self) -> Result<(), FrontEndError> {
        let nodes = self
            .get_node_values_with_scope(self.ast.tree.root().id(), self.syms.get_root_scope_id());

        for (id, value, _enclosing_scope) in nodes.into_iter() {
            if let AstNodeKind::ToProcess(kind) = &value.kind {
                match kind {
                    ToProcessKind::If => {
                        let if_data = Box::new(IfData::new(&self.ast, id));
                        self.change_node_kind(id, AstNodeKind::If(if_data));
                    }

                    ToProcessKind::Application => {
                        let app_data = Box::new(ApplicationData::new(&self.ast, id));
                        self.change_node_kind(id, AstNodeKind::Application(app_data))
                    }

                    ToProcessKind::Let => {
                        panic!()
                    }

                    _ => (),
                };
            }
        }

        Ok(())
    }

    fn create_values(&mut self) -> Result<(), FrontEndError> {
        Ok(())
    }

    fn change_node_kind(&mut self, id: AstNodeId, new_kind: AstNodeKind) {
        let mut sym = self.ast.tree.get_mut(id).unwrap();
        sym.value().kind = new_kind
    }
    fn get_source_file(&self) -> &SourceFile {
        self.ast.get_source_file()
    }

    fn get_source_text(&self, r: &std::ops::Range<usize>) -> &str {
        self.ast.get_source_file().get_text(r).unwrap()
    }
}
