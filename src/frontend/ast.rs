use logos::Source;
use std::{collections::HashMap, default};
use thin_vec::{thin_vec, ThinVec};

use unraveler::{
    alt, is_a, many0, pair, preceded, sep_pair, tag, tuple, wrapped_cut, Collection, Item,
    ParseError,
};

use super::{ploytokens::SlimToken, prelude::*, syntax::SyntaxErrorKind};

use crate::{
    sources::{FileSpan, SourceFile},
    symbols::{ScopeId, SymbolScopeId, SymbolId},
};

#[derive(Clone, PartialEq, Debug)]
pub struct LetData {
    id: AstNodeId,
    let_scope: ScopeId,
    bindings: ThinVec<SymbolId>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct IfData {
    id: AstNodeId,
    predicate: AstNodeId,
    if_true: AstNodeId,
    if_false: Option<AstNodeId>,
}

impl IfData {
    pub fn new(ast: &Ast, id: AstNodeId) -> Self {
        let nth = |n| ast.get_nth_kid_id(id, n);
        let predicate = nth(0).expect("Missing predicate!");
        let if_true = nth(1).expect("Missing true  clause!");
        let if_false = nth(2);

        Self {
            id,
            predicate,
            if_true,
            if_false,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct ApplicationData {
    id: AstNodeId,
    func: AstNodeId,
    args: ThinVec<AstNodeId>,
}

impl ApplicationData {
    pub fn new(ast: &Ast, id: AstNodeId)-> Self {
        let mut kids = ast.tree.get(id).expect("Can't find node").children();
        let func = kids.next().unwrap().id();
        let args = kids.map(|n| n.id()).collect();

        Self {
            id,func,args
        }
    }

    pub fn arrity(&self) -> usize {
        self.args.len()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct LambdaBodyData {
    scope: ScopeId,
    params: ThinVec<SymbolScopeId>,
    returns: (),
}

#[derive(Clone, PartialEq, Debug)]
pub struct LambdaData {
    id: AstNodeId,
    bodies: ThinVec<LambdaBodyData>
}

#[derive(Clone, PartialEq, Debug)]
pub enum ToProcessKind {
    If,
    Application,
    And,
    Or,
    Let,
    Lambda,
    Symbol,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Literal {
    QuotedString(String),
    U8(u8),
    I8(i8),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    F32(f32),
    F64(f64),
    Symbol(SymbolScopeId),
}

impl Into<AstNodeKind> for ToProcessKind {
    fn into(self) -> AstNodeKind {
        AstNodeKind::ToProcess(self)
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum AstNodeKind {
    ToProcess(ToProcessKind),

    If(Box<IfData>),
    Application(Box<ApplicationData>),
    Symbol(SymbolScopeId),
    AssignSymbol(SymbolScopeId),

    BuiltIn,
    Quoted,
    Program,
    Array,
    Number,
    List,

    Null,
    Bool,
    QuotedString,
    Map,
    Pair,
    KeyWordPair,
    Scope,
    KeyWord,
    Define,
    Lambda,
    LambdaBody,
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
    Block,
    #[default]
    Nothing,
    True,
    False,
}

impl AstNodeKind {
    pub fn creates_new_scope(&self) -> bool {
        matches!(self, AstNodeKind::Lambda | AstNodeKind::Let)
    }
}

#[derive(Debug, Clone, Default)]
pub struct AstNode {
    pub kind: AstNodeKind,
    pub token_range: std::ops::Range<usize>,
    pub text_range: std::ops::Range<usize>,
    pub text_span: FileSpan,
}

pub fn get_text_range(tokes_range: &[SlimToken]) -> std::ops::Range<usize> {
    let start_t = &tokes_range.first().unwrap().location.as_range();
    let end_t = &tokes_range.last().unwrap().location.as_range();
    let start = start_t.start;
    let end = end_t.start + end_t.len();
    start..end
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
        let text_span = source_file
            .get_file_span_from_range(text_range.clone())
            .expect("Invalid range");
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
    pub fn get_source_file(&self) -> &SourceFile {
        &self.source_file
    }

    pub fn get_root_id(&self) -> AstNodeId {
        self.tree.root().id()
    }

    pub fn get_nth_kid_id(&self, id: AstNodeId, n: usize) -> Option<AstNodeId> {
        self.tree
            .get(id)
            .unwrap()
            .children()
            .nth(n)
            .map(|node| node.id())
    }

    /// Get all of the ids of this node Recursively, depth first
    fn get_rec_ids_inner(&self, id: AstNodeId, nodes: &mut Vec<AstNodeId>) {
        nodes.push(id);
        let kids = self.tree.get(id).unwrap().children().map(|n| n.id());
        for k in kids {
            self.get_rec_ids_inner(k, nodes)
        }
    }
    pub fn get_rec_ids(&self, id: AstNodeId) -> Vec<AstNodeId> {
        let mut nodes = vec![];
        self.get_rec_ids_inner(id, &mut nodes);
        nodes
    }
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

        if parse_node.meta_data.is_some() {
            let _ = self.meta_data.insert(id, MetaData { node: id });
        }

        for k in parse_node.children.into_iter() {
            self.add_node(Some(id), k)
        }
    }

    pub fn new(parse_node: ParseNode, tokes: &[Token], source_file: SourceFile) -> Self {

        let tokens = tokes
            .iter()
            .filter(|t| t.kind != TokenKind::Comment)
            .map(|t| SlimToken {
                kind: t.kind,
                location: t.location,
                extra: source_file
                    .get_file_span(t.location.start, t.location.len)
                    .expect("Invalid offset"),
            })
            .collect();

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

pub fn to_ast(tokes: &[Token], source_file: SourceFile) -> Result<Ast, FrontEndError> {
    let tokens = Span::from_slice(tokes);

    let (rest, matched) = super::parsers::parse_program(tokens)?;

    if !rest.is_empty() {
        let err = rest.as_slice()[0].location;
        let syntax = SyntaxErrorKind::Unexpected;
        let err = FrontEndError::new(syntax, &err.as_range());
        Err(err)
    } else {
        let ast = Ast::new(matched, tokes, source_file);
        Ok(ast)
    }
}

mod test {
    use super::*;
    fn test() {}
}
