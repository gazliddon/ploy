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

mod prelude {
    pub use super::{
        ast::{to_ast, Ast, AstNode, AstNodeKind, },
        ast::{AstNodeId, AstNodeMut, AstNodeRef},
        error::{FrontEndErrorKind, PResult, FrontEndError},
        ploytokens::tokenize,
    };

    pub use super::ast::AstTree;
    pub use super::parsenode::ParseNode;
    pub use super::ploytokens::Token;
    pub use super::span::Span;
    pub use super::tokens::{ParseText, TokenKind};
}

pub use prelude::*;
