use crate::{
    frontend::{AstNodeId, AstNodeKind, AstNodeRef, AstTree, Module, Type},
    symbols::{ScopeId, SymbolTree},
    value::TypeInfo,
};

use super::instructions::{Instruction, Reg};

pub struct CodeGen<'a> {
    module: &'a Module,
    code: Vec<Instruction>,
}

use std::{process::exit, sync::Arc};

struct Param {
    param_id: u64,
    kind: TypeInfo,
}

pub struct ProcInfo {
    ast_node_id: AstNodeId,
    scope_id: ScopeId,
    params: ThinVec<()>,
}

pub struct DynamicScope {
    node_id: AstNodeId,
    lexical_scope: ScopeId,
    caller: Option<Arc<DynamicScope>>,
    inputs: (),
}

pub struct RegWithType {
    register: usize,
    kind: Type,
}

impl DynamicScope {
    pub fn new(module: &Module, node_id: AstNodeId, caller: Option<Arc<DynamicScope>>) -> Self {
        let lexical_scope = module.get_scope_for_node(node_id).unwrap();

        Self {
            node_id,
            caller,
            lexical_scope,
            inputs: (),
        }
    }
}

use thin_vec::ThinVec;

impl<'a> CodeGen<'a> {
    pub fn new(module: &'a Module) -> Self {
        Self {
            module,
            code: Default::default(),
        }
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

    pub fn emit(&mut self, _i: Instruction) -> usize {
        let addr = self.code.len();
        self.code.push(_i);
        addr
    }
    pub fn emit_at(&mut self, _addr: usize, _i: Instruction) {
        self.code[_addr] = _i;
    }

    pub fn assign(&mut self, _reg: usize, _node_id: AstNodeId) -> Type {
        panic!()
    }

    pub fn get_child_ids(node: AstNodeRef) -> ThinVec<AstNodeId> {
        node.children().map(|n| n.id()).collect()
    }

    pub fn eval_children(&mut self, id: AstNodeId) -> ThinVec<RegWithType> {
        use super::instructions::Instruction::*;

        let children: ThinVec<_> = self
            .node(id)
            .children()
            .map(|n| n.id())
            .enumerate()
            .collect();

        children
            .into_iter()
            .map(|(register, id)| {
                let kind = self.assign(register, id);
                RegWithType { kind, register }
            })
            .collect()
    }

    pub fn eval_n_children(&mut self, _id: AstNodeId, _n: usize) -> ThinVec<RegWithType> {
        panic!()
    }

    pub fn get_pc(&self) -> usize {
        self.code.len()
    }

    pub fn fixup_load(&mut self, addr: usize, new_val: usize) {
        if let Instruction::Load(reg, _) = self.code[addr] {
            self.code[addr] = Instruction::Load(reg, new_val);
        } else {
            panic!()
        }
    }

    pub fn fixup_load_to_pc(&mut self, addr: usize) {
        self.fixup_load(addr, self.get_pc())
    }

    pub fn get_reg(&mut self) -> Reg {
        panic!()
    }

    pub fn new_syms(&mut self) {
        panic!()
    }
    pub fn fixup_syms(&mut self) {}
    pub fn declare_sym(&mut self, _name: &str) {}

    pub fn set_sym(&mut self, _name: &str, _val: usize) {
        panic!()
    }

    pub fn branch_equal(&mut self, _label: &str, _a: Reg, _b: Reg) {
        panic!()
    }

    pub fn jump(&mut self, _label: &str) {
        panic!()
    }
    pub fn label(&mut self, _label: &str) {
        panic!()
    }

    pub fn code_gen(&mut self, node_id: AstNodeId) -> Type {
        use super::instructions::Instruction::*;

        let node = self.node(node_id);
        let _enclosing_scope = self.module.id_to_scope.get(&node_id).unwrap();

        match &node.value().kind {
            AstNodeKind::AssignSymbol(_symbol_id) => {
                // get define result into a register
                // move register to variable
                panic!()
            }

            AstNodeKind::Lambda => {
                panic!()
            }

            AstNodeKind::LambdaBody => {
                panic!()
            }

            AstNodeKind::And => {
                // this is really repeated ifs
                panic!()
            }

            AstNodeKind::Or => {
                panic!()
            }

            AstNodeKind::Let => {
                panic!()
            }

            AstNodeKind::If(..) => {
                let ids: ThinVec<_> = node.children().map(|n| n.id()).collect();

                use super::instructions::Reg::*;

                let p = ids[0];
                let when_true = ids[1];
                let when_false = ids[2];

                let pred_type = self.code_gen(p);

                if pred_type != Type::Bool {
                    self.code_gen(when_false)
                } else {
                    self.new_syms();
                    self.branch_equal("false_clause", Ret(0), Zero);
                    let true_type = self.code_gen(when_true);
                    self.jump("exit");
                    self.label("false_clause");
                    let false_type = self.code_gen(when_false);
                    self.label("exit");
                    self.fixup_syms();

                    if false_type != true_type {
                        panic!("Clauses return different types")
                    }

                    true_type
                }
            }

            AstNodeKind::True => {
                self.emit(Mov(Reg::Ret(0), Reg::Zero));
                Type::Bool
            }

            AstNodeKind::False => {
                self.emit(Load(Reg::Ret(0), 1));
                Type::Bool
            }

            AstNodeKind::Application(..) => {
                let n = node.first_child().unwrap();
                match &n.value().kind {
                    AstNodeKind::Application(..) => {
                        panic!()
                    }

                    AstNodeKind::Symbol(_id) => {
                        // assign a value to each of the params
                        //  Each param is code genned
                        //  compare the types to to type sig of the function

                        // So..
                        // I need a vec of params
                        // each param has an id and a type
                        panic!()
                    }

                    AstNodeKind::BuiltIn => {}

                    _ => panic!(),
                }
                panic!()
            }

            AstNodeKind::BuiltIn => {
                panic!()
            }

            _ => panic!("Unandled node!"),
        }
    }
}
