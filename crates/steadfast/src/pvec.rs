use crate::node::*;
use std::sync::Arc;
use std::fmt::Debug;

const DEFAULT_CHILDREN_PER_NODE: usize = 4;

#[derive(Default,Clone)]
struct PVec<T: Clone, const N: usize = DEFAULT_CHILDREN_PER_NODE> {
    // Maybe replace this with a node?
    node: Arc<Chunk<T, N>>,
}

impl<T> Debug for PVec<T>
where
    T: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PVec")
            .field("size", &self.node.len())
            .finish()
    }
}

impl<T: Clone, const N: usize> From<Chunk<T, N>> for PVec<T, N> {
    fn from(node: Chunk<T, N>) -> Self {
        Self { node: node.into() }
    }
}

impl<T: Clone> PVec<T> {
    pub fn new() -> Self {
        Self::from(Chunk::new())
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<&T> {
        self.node.get(idx)
    }

    pub fn insert_vec(&self, idx: usize, source: Self ) -> Self {
        self.node.insert_chunk(idx, source.node.clone()).into()
    }

    pub fn extend(&self, _extra: Self) -> Self {
        self.node.append_node(Node::Branch(_extra.node)).into()
    }

    pub fn push(&self, value: T) -> Self {
        self.node.append_node(Node::Value(value)).into()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.node.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<T: Clone, const N: usize> PVec<T, N> {
    #[inline]
    fn build(source: Vec<T>) -> Self {
        PVec::from(Chunk::build(source))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]

    fn test_append() { 
        let data: Vec<usize> = vec![0, 1, 2, 3, 4, 5];

        let pv = PVec::build(data);

        for idx in 0..pv.len() {
            println!("trying {idx} {idx:04b}");
            let x = pv.get(idx).cloned();
            println!("{x:?}\n");
        }

        let pv = pv.extend(pv.clone());

        for idx in 0..pv.len() {
            println!("trying {idx} {idx:04b}");
            let x = pv.get(idx).cloned();
            println!("{x:?}\n");
        }

    }

    #[test]
    fn test_pvec() {
        let data: Vec<usize> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let pv = PVec::build(data);

        println!("{pv:?}");

        for idx in 0..pv.len() {
            println!("trying {idx} {idx:04b}");
            let x = pv.get(idx).cloned();
            println!("{x:?}\n");
            assert_eq!(x, Some(idx))
        }
    }
}
