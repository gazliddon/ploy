
use bitvec::prelude::*;
use crate::node::*;
use std::sync::Arc;


const CHILDREN_PER_NODE: usize = 4;
#[derive(Default)]

struct PVec<T: Clone, const N: usize = CHILDREN_PER_NODE> {
    node: Arc<Node<T, N>>,
}

impl<T> std::fmt::Debug for PVec<T>
where
    T: Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PVec")
            .field("size", &self.node.len())
            .finish()
    }
}

impl<T: Clone, const N: usize> From<Node<T, N>> for PVec<T, N> {
    fn from(node: Node<T, N>) -> Self {
        Self { node: node.into() }
    }
}

impl<T: Clone> PVec<T> {
    pub fn new() -> Self {
        Self::from(Node::new())
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        self.node.get(idx)
    }

    pub fn push(&self, _v: T) -> Self {
        panic!()
    }

    pub fn len(&self) -> usize {
        self.node.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

fn make_n_buckets<T: Clone, const N: usize>(n: usize) -> Vec<Arc<Node<T, N>>> {
    let mut dest: Vec<_> = Vec::with_capacity(n);
    for _ in 0..n {
        dest.push(Node::new().into())
    }
    dest
}

impl<T: Clone, const N: usize> PVec<T, N> {
    fn build(source: Vec<T>) -> Self {
        let node = Node::build(source);
        PVec::from(node)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pvec() {
        println!("Node type size is {}", std::mem::size_of::<NodeType<i32,4>>());
        println!("Node is {}", std::mem::size_of::<Node<i32,4>>());
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let pv = PVec::build(data);

        println!("{pv:?}");

        for idx in 0..pv.len() {
            println!("trying {idx} {idx:04b}");
            let x = pv.get(idx).cloned();
            println!("{x:?}\n");
            assert_eq!(x, Some(idx as i32))
        }
    }

    
}
