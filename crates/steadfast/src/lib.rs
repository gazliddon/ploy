#![allow(dead_code)]
#![allow(unused_imports)]

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}

use bitvec::prelude::*;
use std::{
    borrow::BorrowMut,
    cell, default,
    ops::{Deref, Index},
    sync::Arc,
};
use thin_vec::ThinVec;

const BITS_PER_BUCKET: usize = 2;
const CHILDREN_PER_NODE: usize = 1 << BITS_PER_BUCKET;
const MASK: usize = CHILDREN_PER_NODE - 1;

#[derive(Default, Clone)]
enum NodeType<T: Clone> {
    #[default]
    Empty,
    Value(T),
    Node(Arc<Node<T>>),
}

#[derive(Clone)]
struct Node<T: Clone> {
    size: usize,
    bits: BitArray<u8>,
    nodes: [NodeType<T>; CHILDREN_PER_NODE],
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

fn round_up_div(v: usize, divisor: usize) -> usize {
    let add = if v % divisor == 0 { 0 } else { 1 };
    v / divisor + add
}

impl<T: Clone> Default for Node<T> {
    fn default() -> Self {
        Self {
            size: Default::default(),
            bits: Default::default(),
            nodes: Default::default(),
        }
    }
}

impl<T, I> From<I> for Node<T>
where
    I: IntoIterator<Item = T> + ExactSizeIterator<Item = T>,
    T: Clone,
{
    fn from(it: I) -> Self {
        let mut ret = Self::default();
        assert!(it.len() <= CHILDREN_PER_NODE);

        for (i, n) in it.enumerate() {
            ret.nodes[i] = NodeType::<T>::Value(n)
        }
        ret.post_init();
        ret
    }
}

impl<T: Clone> Node<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    fn set_size(&mut self, size: usize) {
        self.size = size;
    }

    fn get_size(&self, idx: usize) -> usize {
        assert!(idx <= CHILDREN_PER_NODE);

        match &self.nodes[idx] {
            NodeType::Empty => 0,
            NodeType::Value(_) => 1,
            NodeType::Node(n) => n.len(),
        }
    }

    fn split_index(&self, index: usize) -> (usize, usize) {
        let mut base = 0;

        for idx in 0..CHILDREN_PER_NODE {
            let cell_size = self.get_size(idx);

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
                Value(_)  | Node(_) => self.bits.set(i, true),
            }
        }
    }

    fn recalc_size(&self) -> usize {

        let mut total = 0;

        for x in &self.nodes {
            total += match x {
                NodeType::Empty => 0,
                NodeType::Value(_) => 1,
                NodeType::Node(n) => n.recalc_size(),
            }
        }

        total
    
    }

    fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            None
        } else {
            let (idx, rest) = self.split_index(index);

            match &self.nodes[idx] {
                NodeType::Value(v) => return Some(v),
                NodeType::Node(n) => n.get(rest),
                NodeType::Empty => panic!(),
            }
        }
    }

    pub fn child_set_node(&mut self, n: usize, val: Arc<Node<T>>) {
        assert!(n < CHILDREN_PER_NODE);
        assert!(self.bits[n] == false);
        self.set_size(self.size + val.size);
        self.nodes[n] = NodeType::Node(val);
        self.bits.set(n, true);
    }

    pub fn child_set_value(&mut self, n: usize, val: T) {
        assert!(n < CHILDREN_PER_NODE);
        assert!(self.bits[n] == false);
        self.nodes[n] = NodeType::Value(val);
        self.bits.set(n, true);
        self.set_size(self.size + 1);
    }
}

#[derive(Default)]
struct PVec<T: Clone> {
    node: Arc<Node<T>>,
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

impl<T: Clone> From<Node<T>> for PVec<T> {
    fn from(node: Node<T>) -> Self {
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

fn make_n_buckets<T: Clone>(n: usize) -> Vec<Arc<Node<T>>> {
    let mut dest: Vec<_> = Vec::with_capacity(n);
    for _ in 0..n {
        dest.push(Node::new().into())
    }
    dest
}

impl<T: Clone> PVec<T> {
    fn build(source: Vec<T>) -> Self {
        if source.len() == 0 {
            PVec::new()
        } else {
            let mut dest = Vec::with_capacity(source.len());

            for c in source.chunks(CHILDREN_PER_NODE) {
                let v: Vec<_> = c.iter().map(|x| x.clone()).collect();
                let n = Node::from(v.into_iter());
                dest.push(n)
            }

            while dest.len() > 1 {

                for d in dest.iter() {
                    print!("{} {} ", d.len(), d.bits)
                }

                let mut new_dest: Vec<Node<T>> = vec![];

                for c in dest.chunks(CHILDREN_PER_NODE).into_iter() {
                    let mut new_node = Node::new();

                    for (i, n) in c.into_iter().enumerate() {
                        new_node.child_set_node(i, n.clone().into())
                    }

                    new_dest.push(new_node);
                }
                dest = new_dest;

                println!("Dest sizes");


                println!("\n");
            }

            PVec::from(dest.first().unwrap().clone())
        }
    }
}

mod test {
    use super::*;

    #[test]
    fn test_vec() {}

    #[test]
    fn test_pvec() {
        println!("Node type size is {}", std::mem::size_of::<NodeType<i32>>());
        println!("Node is {}", std::mem::size_of::<Node<i32>>());
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11,12,13,14,15,16];
        let pv = PVec::build(data);

        println!("{pv:?}");

        for idx in 0..pv.len() {
            println!("trying {idx} {idx:04b}");
            let x = pv.get(idx).cloned();
            println!("{x:?}\n");
            assert_eq!(x, Some(idx as i32))
        }
    }

    #[test]
    fn test_basic_nodes() {
        let mut n0 = Node::new();
        n0.child_set_value(0, 0);
        n0.child_set_value(1, 1);
        n0.child_set_value(2, 2);
        n0.child_set_value(3, 3);

        let mut n1 = Node::new();
        n1.child_set_value(0, 4);
        n1.child_set_value(1, 5);
        n1.child_set_value(2, 6);
        n1.child_set_value(3, 7);

        let mut n2 = Node::new();
        n2.child_set_value(0, 8);
        n2.child_set_value(1, 9);
        n2.child_set_value(2, 10);
        n2.child_set_value(3, 11);

        let mut n_a = Node::new();
        n_a.child_set_node(0, n0.into());
        n_a.child_set_node(1, n1.into());
        n_a.child_set_node(2, n2.into());

        let n_a = Arc::new(n_a);

        for idx in 0..n_a.len() {
            println!("trying {idx} {idx:04b}");
            let x = n_a.get(idx).cloned();
            println!("{x:?}\n");
            assert_eq!(x, Some(idx as i32))
        }
    }
}
