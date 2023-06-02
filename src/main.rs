#![allow(dead_code)]
#![allow(unused_imports)]

mod cli;
mod error;
mod frontend;
mod opts;
mod symbols;
mod value;

use std::ffi::FromVecWithNulError;
use std::num;

use anyhow::Context;
use frontend::{ploytokens::Token, tokens::TokenKind};
use toml::to_string;
use unraveler::Item;

use crate::frontend::ast::AstNodeKind;
use crate::frontend::{ast::to_ast, ploytokens::tokenize};
use thiserror::Error;

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
        .map(|x| (x.get_kind(), get_str(x.clone(), txt).to_string()))
        .collect()
}

use crate::symbols::SymbolTree;

pub struct SyntaxCtx<'a> {
    syms: &'a mut SymbolTree,
    ast: &'a mut frontend::ast::Ast,
    text: &'a str,
}

fn main() -> anyhow::Result<()> {
    let opts = cli::parse_opts()?;

    let mut syms = crate::symbols::SymbolTree::new();

    let program_txt =
        std::fs::read_to_string(&opts.project_file).context("Can't load project file")?;
    let tokes = tokenize(&program_txt);

    let mut ast = to_ast(&tokes)?;
    ast.process(&mut syms, &program_txt)?;

    println!("{:#?}", ast);
    Ok(())
}

use frontend::ast::{Ast, AstNodeRef};
use frontend::error::FrontEndError;

use std::collections::HashMap;

lazy_static::lazy_static! {
    static ref SPESH_TAB : HashMap<&'static str, AstNodeKind> =
        [ ( "define",AstNodeKind::Define ),
        ( "fn",AstNodeKind::Lambda),
        ( "if",AstNodeKind::If),
        ( "let",AstNodeKind::Let),
        ( "cond",AstNodeKind::Cond),
        ( "and",AstNodeKind::And),
        ( "or",AstNodeKind::Or),
        ( "do",AstNodeKind::Do),
        ( "macro",AstNodeKind::Macro), ].into_iter().collect();
}

fn get_special(node: AstNodeRef, source: &str) -> Option<AstNodeKind> {
    let func = node.value();
    let txt = &source[func.text_range.clone()];
    use AstNodeKind::*;

    match txt {
        "define" => Some(Define),
        "fn" => Some(Lambda),
        "if" => Some(If),
        "let" => Some(Let),
        "cond" => Some(Cond),
        "and" => Some(And),
        "or" => Some(Or),
        "do" => Some(Do),
        "macro" => Some(Macro),
        _ => None,
    }
}

impl AstNodeKind {
    pub fn creates_new_scope(&self) -> bool {
        match self {
            AstNodeKind::Lambda | AstNodeKind::Let => true,
            _ => false,
        }
    }

    pub fn is_special(&self) -> bool {
        use AstNodeKind::*;
        match self {
            Define | Lambda | If | Let | Cond | And | Or | Do | Macro => true,
            _ => false,
        }
    }
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

impl Ast {
    pub fn process(&mut self, syms: &mut SymbolTree, source: &str) -> Result<(), FrontEndError> {
        println!("{:#?}", self.tree);
        // self.check_special_forms(syms, source)?;
        self.intern_symbols(syms, source)?;
        self.create_values(syms, source)?;
        Ok(())
    }

    fn create_special_forms(
        &mut self,
        _syms: &mut SymbolTree,
        source: &str,
    ) -> Result<(), FrontEndError> {
        use crate::frontend::ast::AstNodeKind::*;

        // Get all of the ids of nodes that are applications
        let ids: Vec<_> = self
            .tree
            .nodes()
            .filter_map(|n| matches!(n.value().kind, Application).then_some(n.id()))
            .collect();

        // change any of the application nodes that are special forms to be a special form
        for id in ids {
            let k = self.tree.get(id).unwrap().children().next().unwrap();
            let txt = &source[k.value().text_range.clone()];
            if let Some(spesh) = SPESH_TAB.get(txt) {
                let mut node = self.tree.get_mut(k.id()).unwrap();
                node.value().kind = spesh.clone();
            }
        }

        Ok(())
    }

    fn mk_error(&self, _node: AstNodeRef, _err: SyntaxErrorKind) -> SyntaxErrorKind {
        panic!()
    }

    fn check_special_forms(
        &mut self,
        _syms: &mut SymbolTree,
        _source: &str,
    ) -> Result<(), FrontEndError> {
        use AstNodeKind::*;

        let ids: Vec<_> = self
            .tree
            .nodes()
            .filter_map(|n| n.value().kind.is_special().then_some(n.id()))
            .collect();

        for id in ids {
            let node = self.tree.get(id).unwrap();
            let parent = node.parent().unwrap();

            // Check to see if this special form is the first element of an application
            if parent.value().kind != Application  || parent.first_child().unwrap() != node {
                panic!("Special form can only appear at the start of an application");
            }

            let k: Vec<_> = parent.children().collect();
            let args = &k[1..];
            let kind = &node.value().kind;

            let _x: Result<(), SyntaxErrorKind> = match kind {
                Define => check_args(args, 2, 2).and_then(|_| {
                    matches!(args[1].value().kind, Symbol)
                        .then(|| ())
                        .ok_or(SyntaxErrorKind::NotEngoughArgs)
                }),

                Let | Lambda => min_args(args, 1).and_then(|_| {
                    matches!(args[1].value().kind, Array)
                        .then(|| ())
                        .ok_or(SyntaxErrorKind::NotEngoughArgs)
                }),

                If => check_args(args, 2, 3),
                And => min_args(args, 2),
                Or => min_args(args, 2),

                Cond => {
                    panic!()
                }

                Macro => {
                    panic!()
                }

                Do | _ => Ok(()),
            };
        }

        panic!()
    }

    fn intern_symbols(
        &mut self,
        _syms: &mut SymbolTree,
        source: &str,
    ) -> Result<(), FrontEndError> {
        use crate::frontend::ast::AstNodeKind::*;

        for v in self.tree.values() {
            match v.kind {
                Symbol => {
                    let txt = &source[v.text_range.clone()];
                    println!("Symbol is {txt} {:?}", v.text_range);
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
        todo!("create values")
    }
}
