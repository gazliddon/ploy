use crate::{
    frontend::{AstNodeId, AstNodeKind, AstNodeRef, AstTree, Module},
    symbols::SymbolTree,
};

pub struct CodeGen<'a> {
    module: &'a Module,
}

impl<'a> CodeGen<'a> {
    pub fn new(module: &'a Module) -> Self {
        Self { module }
    }

    pub fn tree(&self) -> &AstTree {
        &self.module.ast.tree
    }

    pub fn syms(&self) -> &SymbolTree {
        &self.module.syms
    }

    pub fn node(&self, id: AstNodeId) -> AstNodeRef {
        self.tree().get(id).unwrap()
    }

    pub fn code_gen(&self, node_id: AstNodeId) {
        let node = self.node(node_id);
        use AstNodeKind::*;

        match &node.value().kind {
            SetScope(..) => {}

            Define => {
                // get define result into a register
                // move register to variable
            }

            Application => {}

            BuiltIn => {}

            _ => panic!("Unandled node!"),
        }
    }
}
