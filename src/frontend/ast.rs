use std::{collections::HashMap, default};
use logos::Source;
use thin_vec::{thin_vec, ThinVec};

use unraveler::{
    alt, is_a, many0, pair, preceded, sep_pair, tag, tuple, wrapped_cut, Collection, Item,
    ParseError,
};

use crate::{symbols::{ ScopeId,SymbolScopeId }, sources::{SourceFile, FileSpan}};
use super::{prelude::*, ploytokens::SlimToken};

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
    KeyWordPair,
    Symbol,
    InternedSymbol(SymbolScopeId),
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
    Arg,
    Args,
    LetArg,
    LetArgs,
    SetScope(ScopeId),
    MetaData,
    #[default]
    Nothing,
}

impl AstNodeKind {
    pub fn creates_new_scope(&self) -> bool {
        matches!(self, AstNodeKind::Lambda | AstNodeKind::Let)
    }

    pub fn is_special(&self) -> bool {
        use AstNodeKind::*;
        matches!(
            self,
            Define | Lambda | If | Let | Cond | And | Or | Do | Macro
        )
    }
}

#[derive(Debug, Clone, Default)]
pub struct AstNode {
    pub kind: AstNodeKind,
    pub token_range: std::ops::Range<usize>,
    pub text_range: std::ops::Range<usize>,
    pub text_span : FileSpan,
}

pub fn get_text_range(tokes_range: &[SlimToken]) -> std::ops::Range<usize> {
    let start_t = &tokes_range.first().unwrap().location.as_range();
    let end_t = &tokes_range.last().unwrap().location.as_range();

    let start = start_t.start;
    let end = end_t.start + end_t.len();
    let text_range = start..end;
    text_range
}

impl AstNode {
    pub fn change_kind(&self, new_k: AstNodeKind) -> Self {
        Self {
            kind: new_k,
            ..self.clone()
        }
    }

    fn from_parse_node(node: &ParseNode, tokes: &[SlimToken], source_file: &SourceFile) -> Self {
        let text_range = get_text_range(&tokes[node.range.clone()]);
        let text_span = source_file.get_file_span_from_range(text_range.clone()).expect("Invalid range");
        Self {
            kind: node.kind.clone(),
            token_range: node.range.clone(),
            text_range,
            text_span,
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
    pub source_file: crate::sources::SourceFile,
    pub tokens: Vec<SlimToken>,
}

impl Ast {
    fn add_node(&mut self, parent_id: Option<AstNodeId>, parse_node: ParseNode) {
        let v = AstNode::from_parse_node(&parse_node, &self.tokens, &self.source_file);

        let id = if let Some(parent_id) = parent_id {
            let mut r = self.tree.get_mut(parent_id).unwrap();
            r.append(v).id()
        } else {
            let mut root_nod_mut = self.tree.root_mut();
            let id = root_nod_mut.id();
            *root_nod_mut.value() = v;
            id
        };

        if let Some(_) = parse_node.meta_data {
            let _ = self.meta_data.insert(id, MetaData { node: id });
        }

        for k in parse_node.children.into_iter() {
            self.add_node(Some(id), k)
        }
    }

    pub fn new(parse_node: ParseNode, tokes: &[Token], source_file: SourceFile) -> Self {

        let tokens = tokes.iter().map(|t|SlimToken {
            kind: t.kind.clone(),
            location: t.location.clone(),
            extra: source_file.get_file_span(t.location.start, t.location.len).expect("Invalid offset"),

        }).collect();

        let mut ret = Self {
            tree: AstTree::new(Default::default()),
            meta_data: Default::default(),
            source_file,
            tokens,

        };

        ret.add_node(None, parse_node);

        ret
    }
}

struct UserError {
    error: FrontEndError,
    loc : FileSpan,
}


pub fn to_ast(tokes: &[Token], source_file: SourceFile) -> Result<Ast, FrontEndError> {
    let tokens = Span::from_slice(tokes);

    let (rest, matched) = super::parsers::parse_program(tokens)?;

    if !rest.is_empty() {
        println!("{:?}", rest.as_slice()[0].location);
        panic!("Didn't consume all input");
    }

    let ast = Ast::new(matched, tokes, source_file);

    Ok(ast)
}

mod test {
    use super::*;
    fn test() {}
}
