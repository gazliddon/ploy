/// Compiler front end
/// tokenising and parsing into an AST
mod ast;
mod error;
mod parsenode;
mod parsers;
mod ploytokens;
mod span;
mod syntax;
mod tokens;

mod prelude {
    pub use super::{
        ast::{to_ast, Ast, AstNode, AstNodeKind, },
        ast::{AstNodeId, AstNodeMut, AstNodeRef},
        error::{FrontEndErrorKind, PResult, FrontEndError},
        ploytokens::tokenize,
    };

    pub(crate) use super::ast::AstTree;
    pub(crate) use super::parsenode::ParseNode;
    pub(crate) use super::ploytokens::Token;
    pub(crate) use super::span::Span;
    pub(crate) use super::tokens::{ParseText, TokenKind};
}

pub use prelude::*;
