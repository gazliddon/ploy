use std::{collections::HashMap, default};
use thin_vec::{thin_vec, ThinVec};

use unraveler::{
    alt, is_a, many0, many1, pair, preceded, sep_pair, tag, tuple, wrapped, Collection, Item,
    ParseError,
};

use crate::symbols::ScopeId;

use super::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
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
    KeyWord,
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
    MetaData,
    #[default]
    Nothing,
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

#[derive(Debug, Clone, Default)]
pub struct AstNode {
    pub kind: AstNodeKind,
    pub token_range: std::ops::Range<usize>,
    pub text_range: std::ops::Range<usize>,
}

impl AstNode {
    pub fn change_kind(&self, new_k: AstNodeKind) -> Self {
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

#[derive(Debug, Clone)]
pub struct MetaData {
    node: AstNodeId,
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Debug, Clone)]
pub struct Ast {
    pub tree: AstTree,
    pub meta_data: HashMap<AstNodeId, MetaData>,
}

impl Ast {
    fn add_node(&mut self, parent_id: Option<AstNodeId>, parse_node: ParseNode, tokes: &[Token]) {
        let v = AstNode::from_parse_node(parse_node.clone(), tokes);

        let id = if let Some(parent_id) = parent_id {
            let mut r = self.tree.get_mut(parent_id).unwrap();
            r.append(v).id()
        } else {
            let mut root_nod_mut = self.tree.root_mut();
            let id = root_nod_mut.id();
            *root_nod_mut.value() = v;
            id
        };

        let meta_data = Self::get_meta_data(id, &parse_node);
        self.add_meta_data(id, meta_data);

        for k in parse_node.children.into_iter() {
            self.add_node(Some(id), k, tokes)
        }
    }

    fn get_meta_data(id: AstNodeId, parse_node: &ParseNode) -> Option<MetaData> {
        if let Some(meta_data) = &parse_node.meta_data {
            assert!(meta_data.is_kind(AstNodeKind::MetaData));
            for p in &meta_data.children {
                let (ParseNode{kind: AstNodeKind::Pair,..}, [ParseNode{kind: AstNodeKind::KeyWord,..}, _v] ) = (&p,&p.children[0..2]) else {
                    panic!()
                };
            }
            Some(MetaData { node: id })
        } else {
            None
        }
    }

    fn add_meta_data(&mut self, id: AstNodeId, meta_data: Option<MetaData>) {
        if let Some(meta_data) = meta_data {
            let _ = self.meta_data.insert(id, meta_data);
        }
    }

    fn new(parse_node: ParseNode, tokes: &[Token]) -> Self {
        let mut ret = Self {
            tree: AstTree::new(Default::default()),
            meta_data: Default::default(),
        };

        ret.add_node(None, parse_node, tokes);

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
