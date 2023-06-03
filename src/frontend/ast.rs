use thin_vec::{thin_vec, ThinVec};

use unraveler::{
    alt, is_a, many0, many1, pair, preceded, sep_pair, tag, tuple, wrapped, Collection, Item,
    ParseError,
};

use crate::symbols::ScopeId;

use super::{
    error::FrontEndError,
    parsers::ParseNode,
    tokens::{ParseText, TokenKind},
};

use super::prelude::*;


#[derive(Clone, PartialEq, Debug)]
pub enum AstNodeKind {
    BuiltIn,
    Quoted,
    Program,
    Array,
    Number,
    List,
    Application,
    Null,
    Bool,
    QuotedString,
    Map,
    Pair,
    Symbol,
    InternedSymbol,
    Scope,

    Define,
    Lambda,
    If,
    Let,
    Cond,
    And,
    Or,
    Do,
    Macro,
    SetScope(ScopeId),
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

#[derive(Debug, Clone)]
pub struct AstNode {
    pub kind: AstNodeKind,
    pub token_range: std::ops::Range<usize>,
    pub text_range: std::ops::Range<usize>,
}

impl AstNode {
    pub fn change_kind(&self, new_k : AstNodeKind) -> Self {
        Self {
            kind: new_k,
            ..self.clone()
        }
    }

    fn from_parse_node(node: ParseNode, _tokes: &[Token]) -> Self {
        let tokes_range = &_tokes[node.range.clone()];
        let start_t = &tokes_range.first().unwrap().location.loc;
        let end_t = &tokes_range.last().unwrap().location.loc;

        let start = start_t.start;
        let end = end_t.start + end_t.len;

        Self {
            kind: node.kind,
            token_range: node.range.clone(),
            text_range: start..end,
        }
    }
}

pub type AstTree = ego_tree::Tree<AstNode>;
pub type AstNodeRef<'a> = ego_tree::NodeRef<'a, AstNode>;
pub type AstNodeId = ego_tree::NodeId;
pub type AstNodeMut<'a> = ego_tree::NodeMut<'a, AstNode>;

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct Ast {
    pub tree: AstTree,
}

impl Ast {
    fn add_node(&mut self, parent_id: AstNodeId, parse_node: ParseNode, tokes: &[Token]) {
        let mut r = self.tree.get_mut(parent_id).unwrap();
        let v = AstNode::from_parse_node(parse_node.clone(), tokes);
        let id = r.append(v).id();
        self.add_kids(id, parse_node.children, tokes);
    }

    fn add_kids(&mut self, parent_id: AstNodeId, kids: ThinVec<ParseNode>, tokes: &[Token]) {
        for k in kids.into_iter() {
            self.add_node(parent_id, k, tokes)
        }
    }

    fn new(parse_node: ParseNode, tokes: &[Token]) -> Self {
        let v = AstNode::from_parse_node(parse_node.clone(), tokes);

        let mut ret = Self {
            tree: AstTree::new(v),
        };
        ret.add_kids(ret.tree.root().id(), parse_node.children, tokes);
        ret
    }
}

pub fn to_ast<'a>(tokes: &'a Vec<Token>) -> Result<Ast, PlError> {
    let tokens = Span::new(0, tokes);
    let (rest, matched) = super::parsers::parse_program(tokens)?;

    if !rest.is_empty() {
        println!("{:?}", rest.span[0].location);
        panic!("Didn't consume all input");
    }

    let ast = Ast::new(matched, tokes);

    Ok(ast)
}

mod test {
    use super::*;
    fn test() {}
}
