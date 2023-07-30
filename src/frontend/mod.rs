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
mod module;

mod prelude {
    pub use super::{
        ast::{to_ast, Ast, AstNode, AstNodeKind, ToProcessKind},
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
    pub use super::module::{ Module, ModuleJob };
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

