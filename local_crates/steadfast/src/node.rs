// use bitvec::prelude::*;
use std::sync::Arc;
use thin_vec::ThinVec;

use crate::utils::get_highest_bit;

#[derive(Default, Clone)]
pub(crate) enum Node<T: Clone, const N: usize> {
    #[default]
    Empty,
    Value(T),
    Branch(Arc<Chunk<T, N>>),
}

impl<T: Clone, const N: usize> Node<T, N> {
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

    pub fn get(&self, _idx: usize) -> Option<T> {
        panic!()
    }
}

mod exp {

    use super::*;
    use crate::bitvals::*;

    impl<T, const N: usize> From<ThinVec<Node<T, N>>> for NodeEnum<T, N> {
        fn from(value: ThinVec<Node<T, N>>) -> Self {
            Self::Pointers {
                ptrs: value.into_iter().map(|n| Arc::new(n)).collect(),
            }
        }
    }

    impl<T, const N: usize> From<ThinVec<T>> for NodeEnum<T, N> {
        fn from(value: ThinVec<T>) -> Self {
            Self::Values(value)
        }
    }

    #[derive(Clone)]
    enum NodeEnum<T, const N: usize = 32> {
        Values(ThinVec<T>),
        Pointers { ptrs: ThinVec<Arc<Node<T, N>>> },
    }

    impl<T, const N: usize> NodeEnum<T, N> {
        pub fn calc_len(&self) -> u32 {
            match self {
                Self::Pointers { ptrs, .. } => ptrs.iter().fold(0, |t, v| t + v.len()) as u32,

                Self::Values(vals) => vals.len() as u32,
            }
        }

        pub fn get(&self, idx: usize) -> &T {
            match self {
                Self::Pointers { ptrs, .. } => {
                    let mut lower = 0;
                    for p in ptrs {
                        if idx >= lower && idx < lower + p.len() {
                            return p.get(idx - lower);
                        } else {
                            lower += p.len()
                        }
                    }
                    panic!()
                }
                Self::Values(vals) => vals.get(idx).unwrap(),
            }
        }
    }

    #[derive(Clone)]
    pub struct Node<T, const N: usize> {
        node_enum: Arc<NodeEnum<T, N>>,
        len: u32,
    }

    impl<T, const N: usize> Default for Node<T, N> {
        fn default() -> Self {
            Self {
                node_enum: NodeEnum::Values(Default::default()).into(),
                len: 0,
            }
        }
    }

    impl<T, const N: usize> Node<T, N> {
        #[inline]
        pub fn bitvals() -> BitValues<N> {
            BitValues
        }

        #[inline]
        pub fn len(&self) -> usize {
            self.len as usize
        }

        #[inline]
        pub fn get(&self, idx: usize) -> &T {
            self.node_enum.get(idx)
        }
    }

    impl<T: Clone, const N: usize> From<NodeEnum<T, N>> for Node<T, N> {
        fn from(value: NodeEnum<T, N>) -> Self {
            Self {
                len: value.calc_len(),
                node_enum: value.into(),
                ..Default::default()
            }
        }
    }

    impl<T: Clone, const N: usize, X> From<X> for Node<T, N>
    where
        X: Iterator<Item = T>,
    {
        fn from(into_iter: X) -> Self {
            use itertools::Itertools;

            let mut to_place: ThinVec<Node<T, N>> = thin_vec::thin_vec![];

            for chunk in &into_iter.chunks(N) {
                let c = Node::from(NodeEnum::Values(chunk.into_iter().collect()));
                to_place.push(c)
            }

            while to_place.len() > 1 {
                let mut to_place_next_time: ThinVec<Node<T, N>> = ThinVec::new();

                for c in to_place.chunks(N) {
                    let ptrs: ThinVec<Arc<Node<T, N>>> =
                        c.into_iter().map(|x| Arc::new(x.clone())).collect();
                    let n = Node::from(NodeEnum::Pointers { ptrs });
                    to_place_next_time.push(n)
                }

                to_place = to_place_next_time;
            }

            to_place.into_iter().next().unwrap()
        }
    }

    pub struct TVec<T, const N: usize = 32> {
        node: Arc<Node<T, N>>,
        tail: ThinVec<T>,
        len: u32,
    }

    impl<T, const N: usize> Default for TVec<T, N> {
        fn default() -> Self {
            Self {
                len: 0,
                node: Default::default(),
                tail: ThinVec::with_capacity(N),
            }
        }
    }

    impl<T: Clone, const N: usize, X> From<X> for TVec<T, N>
    where
        X: Iterator<Item = T>,
    {
        fn from(_value: X) -> Self {
            todo!()
        }
    }

    impl<T, const N: usize> TVec<T, N> {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn len(&self) -> usize {
            self.len as usize
        }

        pub fn get(&self, idx: usize) -> Option<&T> {
            if self.len() <= N {
                self.tail.get(idx)
            } else {
                let node_len = self.node.len();
                if node_len < idx {
                    self.tail.get(idx - node_len)
                } else {
                    Some(self.node.get(idx))
                }
            }
        }
    }
}

#[derive(Clone)]
pub(crate) struct Chunk<T: Clone, const N: usize> {
    size: usize,
    nodes: ThinVec<Node<T, N>>,
}

impl<T: Clone, const N: usize> Default for Chunk<T, N> {
    fn default() -> Self {
        let mut x = ThinVec::with_capacity(N);

        for _ in 0..N {
            x.push(Default::default());
        }

        Self {
            size: Default::default(),
            nodes: x,
        }
    }
}

impl<T: Clone, const N: usize> Chunk<T, N> {
    #[inline]
    fn children_per_node() -> usize {
        N
    }

    #[inline]
    fn bits_per_bucket() -> usize {
        get_highest_bit(N).unwrap()
    }
}

impl<T, I, const N: usize> From<I> for Chunk<T, N>
where
    I: IntoIterator<Item = T> + ExactSizeIterator<Item = T>,
    T: Clone,
{
    fn from(it: I) -> Self {
        let mut ret = Self::default();

        assert!(it.len() <= Self::children_per_node());

        for (i, n) in it.enumerate() {
            ret.child_set_node(i, Node::<T, N>::Value(n))
        }

        ret
    }
}

impl<T: Clone, const N: usize> Chunk<T, N> {
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

    pub fn get(&self, index: usize) -> Option<&T> {
        if index >= self.len() {
            None
        } else {
            let (idx, rest) = self.split_index(index);

            match &self.nodes[idx] {
                Node::Value(v) => return Some(v),
                Node::Branch(n) => n.get(rest),
                Node::Empty => panic!(),
            }
        }
    }

    pub fn child_set_node(&mut self, n: usize, val: Node<T, N>) {
        assert!(n < Self::children_per_node());
        let size_to_gain = val.len();
        let size_to_lose = self.nodes[n].len();
        self.size += size_to_gain - size_to_lose;
        self.nodes[n] = val
    }

    fn child_insert_node(&self, n: usize, val: Node<T, N>) -> Self {
        assert!(n <= Self::children_per_node());
        let mut ret = self.clone();

        if ret.nodes[n].is_empty() {
            ret.child_set_node(n, val)
        } else {
            let mut child = Chunk::new();
            child.child_set_node(0, val);
            child.child_set_node(1, ret.nodes[n].clone());
            ret.child_set_node(n, Node::Branch(child.into()))
        }
        ret
    }

    fn get_append_index(&self) -> Option<usize> {
        let mut lowest = None;
        // Find an unused slot at end of data
        for idx in N - 1..=0 {
            if self.nodes.is_empty() {
                lowest = Some(idx)
            } else {
                break;
            }
        }
        lowest
    }

    pub(crate) fn append_node(&self, val: Node<T, N>) -> Self {
        let mut ret = self.clone();

        if let Some(idx) = self.get_append_index() {
            ret.child_set_node(idx, val)
        } else {
            let mut child = Chunk::new();
            child.child_set_node(0, ret.nodes.last().unwrap().clone());
            child.child_set_node(1, val);
            ret.child_set_node(N - 1, Node::Branch(child.into()))
        }

        ret
    }

    fn child_insert_value(&self, n: usize, val: T) -> Self {
        let val = Node::Value(val);
        self.child_insert_node(n, val)
    }

    pub(crate) fn insert_value(&self, index: usize, v: T) -> Self {
        self.insert_node(index, Node::Value(v))
    }
    pub(crate) fn insert_chunk(&self, index: usize, node: Arc<Self>) -> Self {
        self.insert_node(index, Node::Branch(node))
    }

    pub(crate) fn insert_node(&self, index: usize, node: Node<T, N>) -> Self {
        let (idx, rest) = self.split_index(index);

        match &self.nodes[idx] {
            Node::Empty => {
                let mut ret = self.clone();
                ret.child_set_node(idx, node);
                ret
            }
            Node::Value(..) => {
                // if theres a value I need to insert at the value
                self.child_insert_node(idx, node)
            }
            Node::Branch(n) => {
                let mut ret = self.clone();
                ret.child_set_node(idx, Node::Branch(n.insert_node(rest, node).into()));
                ret
            }
        }
    }

    pub fn build(source: Vec<T>) -> Self {
        if source.len() == 0 {
            Self::new()
        } else {
            let mut dest: Vec<Chunk<T, N>> = Vec::with_capacity(source.len());

            for c in source.chunks(Self::children_per_node()) {
                let v: Vec<_> = c.iter().map(|x| x.clone()).collect();
                let n: Chunk<T, N> = Chunk::from(v.into_iter());
                dest.push(n)
            }

            while dest.len() > 1 {
                let mut new_dest: Vec<Chunk<T, N>> = vec![];

                for c in dest.chunks(Self::children_per_node()).into_iter() {
                    let mut new_node: Chunk<T, N> = Chunk::new();

                    for (i, n) in c.into_iter().enumerate() {
                        let nt = Node::<T, N>::Branch(n.clone().into());
                        new_node.child_set_node(i, nt)
                    }
                    new_dest.push(new_node);
                }

                dest = new_dest;
            }

            dest.into_iter().next().unwrap()
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

        let n0 = Chunk::<_, 4>::build(pre_insert.clone());
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
    fn test_insert() {
        println!("Low level insert testing");
        let pre_insert = vec![0, 1, 2, 3, 255, 255, 255];
        let _post_insert = vec![0, 100, 1, 2, 3];

        let node = Chunk::<_, 4>::build(pre_insert);

        for i in 0..node.len() {
            println!("{:?}", node.get(i))
        }
        println!("");

        let node2 = node.insert_chunk(1, node.clone().into());

        for i in 0..node2.len() {
            println!("{:?}", node2.get(i))
        }

        println!("");
    }

    #[test]
    fn test_basic_nodes() {
        use Node::*;

        let mut n0: Chunk<i32, 4> = Chunk::new();
        n0.child_set_node(0, Value(0));
        n0.child_set_node(1, Value(1));
        n0.child_set_node(2, Value(2));
        n0.child_set_node(3, Value(3));

        let mut n1 = Chunk::new();
        n1.child_set_node(0, Value(4));
        n1.child_set_node(1, Value(5));
        n1.child_set_node(2, Value(6));
        n1.child_set_node(3, Value(7));

        let mut n2 = Chunk::new();
        n2.child_set_node(0, Value(8));
        n2.child_set_node(1, Value(9));
        n2.child_set_node(2, Value(10));
        n2.child_set_node(3, Value(11));

        let mut n_a = Chunk::new();
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
