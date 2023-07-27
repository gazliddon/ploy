use std::ops::RangeFrom;

use super::prelude::*;
use thin_vec::{thin_vec, ThinVec};

pub struct Building;
pub struct Built;

#[derive(Clone, PartialEq, Debug)]
pub struct ParseNode {
    pub kind: AstNodeKind,
    pub range: std::ops::Range<usize>,
    pub children: ThinVec<ParseNode>,
    pub meta_data: Option<Box<ParseNode>>,
}

impl ParseNode {

    pub fn builder<K: Into<AstNodeKind>>(kind: K, input: Span, rest: Span) -> NodeBuilder {
        NodeBuilder::from_spans(kind.into(), input, rest)
    }

    pub fn change_kind<K: Into<AstNodeKind>>(mut self, kind: K) -> Self {
        self.kind = kind.into();
        self
    }
    pub fn change_meta(mut self, meta: Option<ParseNode>) -> Self {
        self.meta_data = meta.map(|m| m.into());
        self
    }

    pub fn is_kind(&self, k: AstNodeKind) -> bool {
        self.kind == k
    }

    pub fn get_tokes_span<'a, T>(&self, tokes: &'a [T]) -> &'a [T] {
        &tokes[self.range.clone()]
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct NodeBuilder {
    pub kind: AstNodeKind,
    pub range: std::ops::Range<usize>,
    pub children: ThinVec<ParseNode>,
    pub meta_data: Option<ParseNode>,
}

impl From<NodeBuilder> for ParseNode {
    fn from(value: NodeBuilder) -> Self {
        value.build()
    }
}

impl NodeBuilder {
    pub fn new(kind: AstNodeKind, start: usize, len: usize) -> Self {
        Self {
            kind,
            range: start..start + len,
            children: thin_vec![],
            meta_data: None,
        }
    }

    pub fn from_spans(kind: AstNodeKind, input: Span, rest: Span) -> Self {
        let input = input.get_range();
        let rest = rest.get_range();
        let range = input.start..rest.start;

        Self::new(kind, range.start, range.len())
    }


    pub fn child(mut self, k: ParseNode) -> Self {
        self.children.push(k);
        self
    }

    pub fn meta(mut self, meta_data: ParseNode) -> Self {
        self.meta_data = Some(meta_data);
        self
    }

    pub fn meta_opt(mut self, meta_data: Option<ParseNode>) -> Self {
        self.meta_data = meta_data;
        self
    }

    pub fn children<X: Into<ThinVec<ParseNode>>>(mut self, children: X) -> Self {
        let tvec: ThinVec<_> = children.into();
        self.children.extend(tvec);
        self
    }

    pub fn kind(mut self, kind: AstNodeKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn build(self) -> ParseNode {
        ParseNode {
            kind: self.kind,
            range: self.range,
            children: self.children,
            meta_data: self.meta_data.map(Box::new),
        }
    }
}
