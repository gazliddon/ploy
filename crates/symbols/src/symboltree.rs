use std::collections::HashMap;
use thin_vec::ThinVec;
use super::prelude::*;

use super::{
    symboltable::SymbolTable, symboltreereader::SymbolTreeReader,
    symboltreewriter::SymbolTreeWriter, 
};

////////////////////////////////////////////////////////////////////////////////
// SymbolTree
type SymbolTreeTree<SCOPEID, SYMID> = ego_tree::Tree<SymbolTable<SCOPEID, SYMID>>;
type SymbolNodeRef<'a, SCOPEID, SYMID> = ego_tree::NodeRef<'a, SymbolTable<SCOPEID, SYMID>>;
type SymbolNodeId = ego_tree::NodeId;
type SymbolNodeMut<'a, SCOPEID, SYMID> = ego_tree::NodeMut<'a, SymbolTable<SCOPEID, SYMID>>;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SymbolTree<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: Clone,
{
    pub (crate) tree: ego_tree::Tree<SymbolTable<SCOPEID, SYMID>>,
    pub (crate) root_scope_id: SCOPEID,
    pub (crate) next_scope_id: SCOPEID,
    pub (crate) scope_id_to_node_id: HashMap<SCOPEID, SymbolNodeId>,
    pub (crate) scope_id_to_symbol_info:
        HashMap<SymbolScopeId<SCOPEID, SYMID>, SymbolInfo<SCOPEID, SYMID, SYMVALUE>>,
}

struct SymbolNode<SCOPEID, SYMID>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
{
    parent: Option<SCOPEID>,
    children: HashMap<String, SymbolTable<SCOPEID, SYMID>>,
}

impl<SCOPEID, SYMID, SYMVALUE> Default for SymbolTree<SCOPEID, SYMID, SYMVALUE>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    SYMVALUE: Clone,
{
    fn default() -> Self {
        let root: SymbolTable<SCOPEID, SYMID> =
            SymbolTable::new("", "", 0.into(), SymbolResolutionBarrier::default());
        let tree: SymbolTreeTree<SCOPEID, SYMID> = SymbolTreeTree::new(root);
        let current_scope = tree.root().id();
        let mut scope_id_to_node_id: HashMap<SCOPEID, SymbolNodeId> = Default::default();
        scope_id_to_node_id.insert(0.into(), current_scope);

        Self {
            tree,
            root_scope_id: 0.into(),
            next_scope_id: 1.into(),
            scope_id_to_node_id,
            scope_id_to_symbol_info: Default::default(),
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Uses ego tree
impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    V: Clone,
{
    pub fn get_node_id_from_scope_id(
        &self,
        scope_id: SCOPEID,
    ) -> Result<SymbolNodeId, SymbolError<SCOPEID, SYMID>> {
        self.scope_id_to_node_id
            .get(&scope_id)
            .cloned()
            .ok_or(SymbolError::InvalidScope)
    }

    fn get_node_from_id(
        &self,
        scope_id: SCOPEID,
    ) -> Result<SymbolNodeRef<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;
        self.tree.get(node_id).ok_or(SymbolError::InvalidScope)
    }

    pub fn get_parent_scope_id(&self, scope_id: SCOPEID) -> Option<SCOPEID> {
        let node = self.get_node_from_id(scope_id).expect("Illegal scope id");
        node.parent().map(|n| n.value().get_scope_id())
    }

    fn children(
        &self,
        scope_id: SCOPEID,
    ) -> impl Iterator<Item = &SymbolTable<SCOPEID, SYMID>> + '_ {
        let node = self.get_node_from_id(scope_id).unwrap();
        node.children().map(|n| n.value())
    }


    pub(crate) fn get_scope(
        &self,
        scope_id: SCOPEID,
    ) -> Result<&SymbolTable<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        self.get_node_from_id(scope_id).map(|n| n.value())
    }


    fn on_value_mut<F, R>(
        &mut self,
        scope_id: SCOPEID,
        mut f: F,
    ) -> Result<R, SymbolError<SCOPEID, SYMID>>
    where
        F: FnMut(&mut SymbolTable<SCOPEID, SYMID>) -> Result<R, SymbolError<SCOPEID, SYMID>>,
    {
        let node_id = self.get_node_id_from_scope_id(scope_id)?;

        if let Some(ref mut node_mut) = self.tree.get_mut(node_id) {
            f(node_mut.value())
        } else {
            Err(SymbolError::InvalidId)
        }
    }

    pub(crate) fn insert_new_table(
        &mut self,
        name: &str,
        parent_id: SCOPEID,
        barrier: SymbolResolutionBarrier,
    ) -> SCOPEID {
        let tab = self.create_new_table(name, parent_id, barrier);
        let tab_id = tab.get_scope_id();
        let parent_id = self.scope_id_to_node_id.get(&parent_id).unwrap();
        let mut parent_mut = self.tree.get_mut(*parent_id).unwrap();
        let mut n = parent_mut.append(tab);
        self.scope_id_to_node_id.insert(tab_id, n.id());
        n.value().get_scope_id()
    }
}

////////////////////////////////////////////////////////////////////////////////
// Scope management
impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    V: Clone,
{
    pub fn set_symbol_for_id(
        &mut self,
        symbol_id: SymbolScopeId<SCOPEID, SYMID>,
        val: V,
    ) -> Result<(), SymbolError<SCOPEID, SYMID>> {
        self.on_symbol_mut(symbol_id, |si| {
            si.value = Some(val.clone());
            Ok(())
        })
    }

    pub fn remove_symbol_for_id(
        &mut self,
        name: &str,
        scope_id: SCOPEID,
    ) -> Result<(), SymbolError<SCOPEID, SYMID>> {
        self.on_value_mut(scope_id, |syms| syms.remove_symbol(name))
    }
    fn on_symbol_mut<F, R>(
        &mut self,
        id: SymbolScopeId<SCOPEID, SYMID>,
        mut f: F,
    ) -> Result<R, SymbolError<SCOPEID, SYMID>>
    where
        F: FnMut(&mut SymbolInfo<SCOPEID, SYMID, V>) -> Result<R, SymbolError<SCOPEID, SYMID>>,
    {
        let x = self
            .scope_id_to_symbol_info
            .get_mut(&id)
            .ok_or(SymbolError::NotFound)?;
        f(x)
    }
    pub fn set_value_for_id(
        &mut self,
        id: SymbolScopeId<SCOPEID, SYMID>,
        val: V,
    ) -> Result<(), SymbolError<SCOPEID, SYMID>> {
        self.on_symbol_mut(id, move |sym| {
            sym.value = Some(val.clone());
            Ok(())
        })
    }
    pub fn add_reference_symbol(
        &mut self,
        name: &str,
        scope_id: SCOPEID,
        symbol_id: SymbolScopeId<SCOPEID, SYMID>,
    ) -> Result<(), SymbolError<SCOPEID, SYMID>> {
        self.on_value_mut(scope_id, |syms| syms.add_reference_symbol(name, symbol_id))
    }
    pub fn create_symbol_in_scope(
        &mut self,
        scope_id: SCOPEID,
        name: &str,
    ) -> Result<SymbolScopeId<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        let (si, symbol_id) = self.on_value_mut(scope_id, |syms| {
            let symbol_id = syms.create_symbol(name)?;
            let si = SymbolInfo::new(name, None, symbol_id, syms.get_scope_fqn_name());
            Ok((si, symbol_id))
        })?;

        self.scope_id_to_symbol_info.insert(symbol_id, si);
        Ok(symbol_id)
    }

    pub fn resolve_label(
        &self,
        name: &str,
        scope_id: SCOPEID,
        barrier: SymbolResolutionBarrier,
    ) -> Result<SymbolScopeId<SCOPEID, SYMID>, SymbolError<SCOPEID, SYMID>> {
        let mut node_scope_id = Some(scope_id);

        while let Some(n) = node_scope_id {
            let v = self.get_scope(n).unwrap();

            if let Ok(exists) = v.get_symbol_id(name) {
                return Ok(exists);
            }

            if !v.get_symbol_resoultion_barrier().can_pass_barrier(barrier) {
                break;
            }
            node_scope_id = self.get_parent_scope_id(n);
        }

        Err(SymbolError::NotFound)
    }
    pub fn get_symbol_info_from_scoped_name(
        &self,
        name: &ScopedName,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError<SCOPEID, SYMID>> {
        assert!(name.is_abs());

        let scopes = name.path();
        let name = name.symbol();

        let mut current_node = self.get_root_scope_id();

        let mut found = false;

        for path_part in scopes.iter() {
            for c in self.children(current_node) {
                if c.get_scope_name() == *path_part {
                    current_node = c.get_scope_id();
                    found = true;
                    break;
                }
            }

            if found == false {
                return Err(SymbolError::NotFound);
            }
        }

        self.get_symbol_info(name, current_node)
    }
    pub fn get_root_scope_id(&self) -> SCOPEID {
        self.root_scope_id
    }

    pub fn create_or_get_scope_for_parent(&mut self, name: &str, id: SCOPEID) -> SCOPEID {
        for v in self.children(id) {
            if v.get_scope_name() == name {
                let id = v.get_scope_id();
                return id;
            }
        }
        self.insert_new_table(name, id, SymbolResolutionBarrier::default())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Public functions
impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    V: Clone,
{
    pub fn get_sub_scope_id(
        &self,
        name: &str,
        scope_id: SCOPEID,
    ) -> Result<SCOPEID, SymbolError<SCOPEID, SYMID>> {
        let name = ScopedName::new(name);
        assert!(name.is_relative());
        let path = name.path();
        self.find_sub_scope_id(path, scope_id)
    }

    pub fn get_scope_id(&self, name: &str) -> Result<SCOPEID, SymbolError<SCOPEID, SYMID>> {
        let name = ScopedName::new(name);
        assert!(name.is_abs());
        let scope_id = self.get_root_scope_id();
        let path = name.path();
        self.find_sub_scope_id(path, scope_id)
    }
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_sub_scope_id(
        &self,
        path: &[&str],
        scope_id: SCOPEID,
    ) -> Result<SCOPEID, SymbolError<SCOPEID, SYMID>> {
        let mut current_node = scope_id;

        for path_part in path {
            let mut found = false;

            for k in self.children(current_node) {
                if path_part == &k.get_scope_name() {
                    found = true;
                    current_node = k.get_scope_id();
                }
            }

            if !found {
                return Err(SymbolError::NoValue);
            }
        }

        Ok(current_node)
    }

    pub fn create_symbols_in_scope(
        &mut self,
        scope_id: SCOPEID,
        names: &[String],
    ) -> Result<ThinVec<SymbolScopeId<SCOPEID, SYMID>>, SymbolError<SCOPEID, SYMID>> {
        let ret: Result<ThinVec<SymbolScopeId<SCOPEID, SYMID>>, SymbolError<SCOPEID, SYMID>> =
            names
                .iter()
                .map(|name| self.create_symbol_in_scope(scope_id, name))
                .collect();
        ret
    }

    pub fn scope_exists(&self, scope: SCOPEID) -> bool {
        self.scope_id_to_node_id.get(&scope).is_some()
    }

    pub fn get_fqn_from_id(&self, scope_id: SCOPEID) -> String {
        let scope = self.get_scope(scope_id).expect("Invalid scope");
        scope.get_scope_fqn_name().to_owned()
    }

    pub fn get_writer(&mut self, scope_id: SCOPEID) -> SymbolTreeWriter<SCOPEID, SYMID, V> {
        SymbolTreeWriter::new(self, scope_id)
    }

    pub fn get_root_writer(&mut self) -> SymbolTreeWriter<SCOPEID, SYMID, V> {
        SymbolTreeWriter::new(self, self.get_root_scope_id())
    }

    pub fn get_reader(&self, scope_id: SCOPEID) -> SymbolTreeReader<SCOPEID, SYMID, V> {
        SymbolTreeReader::new(self, scope_id)
    }

    pub fn get_root_reader(&self) -> SymbolTreeReader<SCOPEID, SYMID, V> {
        self.get_reader(self.get_root_scope_id())
    }

    pub fn get_symbol_info_from_id(
        &self,
        symbol_id: SymbolScopeId<SCOPEID, SYMID>,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError<SCOPEID, SYMID>> {
        self.scope_id_to_symbol_info
            .get(&symbol_id)
            .ok_or(SymbolError::InvalidId)
    }

    pub fn get_symbol_info_from_name(
        &self,
        name: &str,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError<SCOPEID, SYMID>> {
        let name = ScopedName::new(name);
        self.get_symbol_info_from_scoped_name(&name)
    }

    pub fn get_symbol_info(
        &self,
        name: &str,
        scope_id: SCOPEID,
    ) -> Result<&SymbolInfo<SCOPEID, SYMID, V>, SymbolError<SCOPEID, SYMID>> {
        let n = self.get_scope(scope_id)?;
        let id = n.get_symbol_id(name)?;
        self.scope_id_to_symbol_info
            .get(&id)
            .ok_or(SymbolError::NotFound)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Private implementation funcs
impl<SCOPEID, SYMID, V> SymbolTree<SCOPEID, SYMID, V>
where
    SCOPEID: ScopeIdTraits,
    SYMID: SymIdTraits,
    V: Clone,
{
    pub fn get_and_inc_next_scope_id(&mut self) -> SCOPEID {
        let ret = self.next_scope_id;
        self.next_scope_id += 1;
        ret.into()
    }

    pub fn get_next_scope_id(&self) -> SCOPEID {
        self.next_scope_id
    }

    fn create_new_table(
        &mut self,
        name: &str,
        parent_id: SCOPEID,
        barrier: SymbolResolutionBarrier,
    ) -> SymbolTable<SCOPEID, SYMID> {
        let parent_fqn = self.get_fqn_from_id(parent_id);
        let fqn = format!("{parent_fqn}::{name}");
        let scope_id = self.get_and_inc_next_scope_id();
        SymbolTable::new(name, &fqn, scope_id, barrier)
    }
}

#[allow(unused_imports)]
mod test {
    use super::*;

    #[test]
    fn test_sym_tree() {
        // let mut st = SymbolTree::default();

        // let _ = st.add_symbol_with_value("root_gaz", 100);

        // st.set_current_scope("scope_a");
        // let _ = st.add_symbol_with_value("gaz", 100);
        // let _ = st.add_symbol_with_value("root_gaz", 100);

        // let scope_fqn = st.get_current_scope_fqn();
        // println!("SCOPE is {scope_fqn}");
        // st.pop_scope();

        // let scope_fqn = st.get_current_scope_fqn();
        // println!("SCOPE is {scope_fqn}");
    }
}
