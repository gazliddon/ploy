use bitvec::prelude::*;
use std::{
    borrow::BorrowMut,
    ops::{Deref, Index},
    sync::Arc,
};

use thin_vec::ThinVec;

#[derive(Default, Clone)]
pub (crate) enum NodeType<T: Clone, const N: usize> {
    #[default]
    Empty,
    Value(T),
    Branch(Arc<Node<T, N>>),
}

impl<T: Clone, const N: usize> NodeType<T, N> {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            _ => false,
        }
    }
    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Value(..) => 1,
            Self::Branch(n) => n.len(),
        }
    }
}

#[derive(Clone)]
pub (crate) struct Node<T: Clone, const N: usize> {
    size: usize,
    bits: BitArray<u8>,
    nodes: ThinVec<NodeType<T,N>>,
}

fn get_highest_bit(v: usize) -> Option<usize> {
    let bits = std::mem::size_of::<usize>() * 8;
    for i in (0..bits).rev() {
        if v & 1 << i != 0 {
            return Some(i);
        }
    }
    None
}

impl<T: Clone, const N: usize> Default for Node<T, N> {
    fn default() -> Self {

        let mut x = ThinVec::with_capacity(N);

        for _ in 0..N  {
            x.push(Default::default());
        }

        Self {
            size: Default::default(),
            bits: Default::default(),
            nodes: x,
        }
    }
}

impl<T: Clone, const N: usize> Node<T, N> {
    #[inline]
    fn children_per_node() -> usize {
        N
    }

    #[inline]
    fn bits_per_bucket() -> usize {
        get_highest_bit(N).unwrap()
    }
}

impl<T, I, const N: usize> From<I> for Node<T, N>
where
    I: IntoIterator<Item = T> + ExactSizeIterator<Item = T>,
    T: Clone,
{
    fn from(it: I) -> Self {
        let mut ret = Self::default();

        assert!(it.len() <= Self::children_per_node());

        for (i, n) in it.enumerate() {
            ret.child_set_node(i, NodeType::<T, N>::Value(n))
        }

        ret
    }
}

impl<T: Clone, const N: usize> Node<T, N> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    fn set_size(&mut self, size: usize) {
        self.size = size;
    }

    fn split_index(&self, index: usize) -> (usize, usize) {
        let mut base = 0;

        for (idx, n) in self.nodes.iter().enumerate() {
            let cell_size = n.len();

            if cell_size > 0 {
                let r = base..(base + cell_size);

                if r.contains(&index) {
                    return (idx, index - base);
                }
                base = base + cell_size;
            }
        }

        panic!()
    }

    fn post_init(&mut self) {
        self.recalc_bits();
        self.size = self.recalc_size();
    }

    fn recalc_bits(&mut self) {
        use NodeType::*;

        for (i, n) in self.nodes.iter_mut().enumerate() {
            match n {
                Empty => self.bits.set(i, false),
                Value(_) | Branch(_) => self.bits.set(i, true),
            }
        }
    }

    fn recalc_size(&self) -> usize {
        let mut total = 0;
        for x in &self.nodes {
            total += x.len();
        }
        total
    }

    pub  fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            None
        } else {
            let (idx, rest) = self.split_index(index);

            match &self.nodes[idx] {
                NodeType::Value(v) => return Some(v),
                NodeType::Branch(n) => n.get(rest),
                NodeType::Empty => panic!(),
            }
        }
    }

    pub fn child_set_node(&mut self, n: usize, val: NodeType<T, N>) {
        assert!(n < Self::children_per_node());
        let size_to_gain = val.len();
        let not_empty = !val.is_empty();

        let size_to_lose = self.nodes[n].len();
        self.size += size_to_gain - size_to_lose;
        self.bits.set(n, not_empty);
        self.nodes[n] = val
    }

    fn child_insert_value(&self, n: usize, val: T) -> Self {
        assert!(n <= Self::children_per_node());
        let mut ret = self.clone();
        let val = NodeType::Value(val);

        if ret.nodes[n].is_empty() {
            ret.child_set_node(n, val)
        } else {
            let mut child = Node::new();
            child.child_set_node(0, val);
            child.child_set_node(1, ret.nodes[n].clone());
            ret.child_set_node(n, NodeType::Branch(child.into()))
        }
        ret
    }

    pub fn build(source: Vec<T>) -> Self {
        if source.len() == 0 {
            Self::new()
        } else {
            let mut dest: Vec<Node<T, N>> = Vec::with_capacity(source.len());

            for c in source.chunks(Self::children_per_node()) {
                let v: Vec<_> = c.iter().map(|x| x.clone()).collect();
                let n: Node<T, N> = Node::from(v.into_iter());
                dest.push(n)
            }

            while dest.len() > 1 {
                let mut new_dest: Vec<Node<T, N>> = vec![];

                for c in dest.chunks(Self::children_per_node()).into_iter() {
                    let mut new_node: Node<T, N> = Node::new();

                    for (i, n) in c.into_iter().enumerate() {
                        let nt = NodeType::<T, N>::Branch(n.clone().into());
                        new_node.child_set_node(i, nt)
                    }
                    new_dest.push(new_node);
                }
                dest = new_dest;
            }

            dest.first().unwrap().clone()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_pvec_insert() {}

    #[test]
    fn test_low_level_insert_val() {
        println!("Low level insert testing");
        let pre_insert = vec![0, 1, 2, 3];
        let post_insert = vec![0, 100, 1, 2, 3];

        let mut n0 = Node::<_,4>::new();

        for i in 0..pre_insert.len() {
            n0.child_set_node(i, NodeType::Value(i));
        }

        println!("Pre-insert");

        for i in 0..n0.len() {
            println!("{:?}", n0.get(i))
        }
        println!("");

        let n2 = n0.child_insert_value(1, 100);
        println!("Post insert");

        for i in 0..n2.len() {
            println!("{:?}", n2.get(i))
        }
        println!("");

        println!("Checking");
        for (i, v) in post_insert.into_iter().enumerate() {
            let desired = Some(v);
            let got = n2.get(i).cloned();
            println!("{i} = {:?} {:?}", got, v);
            assert_eq!(got, desired);
        }

        println!("");
        println!("");
    }

    #[test]
    fn test_basic_nodes() {
        use NodeType::*;

        let mut n0 : Node<i32,4> = Node::new();
        n0.child_set_node(0, Value(0));
        n0.child_set_node(1, Value(1));
        n0.child_set_node(2, Value(2));
        n0.child_set_node(3, Value(3));

        let mut n1 = Node::new();
        n1.child_set_node(0, Value(4));
        n1.child_set_node(1, Value(5));
        n1.child_set_node(2, Value(6));
        n1.child_set_node(3, Value(7));

        let mut n2 = Node::new();
        n2.child_set_node(0, Value(8));
        n2.child_set_node(1, Value(9));
        n2.child_set_node(2, Value(10));
        n2.child_set_node(3, Value(11));

        let mut n_a = Node::new();
        n_a.child_set_node(0, Branch(n0.into()));
        n_a.child_set_node(1, Branch(n1.into()));
        n_a.child_set_node(2, Branch(n2.into()));

        let n_a = Arc::new(n_a);

        for idx in 0..n_a.len() {
            println!("trying {idx} {idx:04b}");
            let x = n_a.get(idx).cloned();
            println!("{x:?}\n");
            assert_eq!(x, Some(idx as i32))
        }
    }
}
