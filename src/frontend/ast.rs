use thin_vec::{thin_vec, ThinVec};
use unraveler::{
    alt, is_a, many0, many1,pair, preceded, sep_pair, tag, tuple, wrapped, Collection, Item, ParseError,
};

use super::{
    error::{FrontEndError, PResult, PlError},
    ploytokens::Token,
    tokens::{ParseText, TokenKind},
    Span, parsers::ParseNode,
};

#[derive(Clone, PartialEq, Debug)]
pub enum AstNodeKind {
    Special,
    Quoted,
    Lambda,
    Program,
    Array,
    Number,
    List,
    Application,
    Null,
    Bool,
    QuotedString,
    Symbol,
    InternedSymbol,
    Map,
    Pair,
}

pub struct AstNode {
    kind: AstNodeKind,
    token_range: std::ops::Range<usize>,
    text_range: std::ops::Range<usize>,
}

pub type AstTree = ego_tree::Tree<AstNode>;
pub type AstNodeRef<'a> = ego_tree::NodeRef<'a, AstNode>;
pub type AstNodeId = ego_tree::NodeId;
pub type AstNodeMut<'a> = ego_tree::NodeMut<'a, AstNode>;


////////////////////////////////////////////////////////////////////////////////
pub struct Ast {
    source: String,
    tree: AstTree,
}

pub fn to_ast<'a>(tokes: &'a Vec<Token>) -> PResult<'a, ParseNode> {
    let tokens = Span::new(0, tokes);
    let (rest, matched) = super::parsers::parse_program(tokens)?;
    Ok((rest, matched))
}

mod test {
    use super::*;
    fn test() {}
}
