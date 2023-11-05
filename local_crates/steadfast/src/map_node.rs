use std::sync::Arc;
use thin_vec::ThinVec;

use super::utils::get_highest_bit;

trait BitValuesTrait {
    const NON_ZERO: ();
    const BITS: u32;
    const MASK: u32;
}

struct BitValues<const NUM_OF_BITS: usize>{
}

impl<const NUM_OF_BITS: usize> BitValuesTrait for BitValues<NUM_OF_BITS> {
    const NON_ZERO: () = if NUM_OF_BITS == 0 || NUM_OF_BITS >= 32 {panic!("Bits must be 1..32")};
    const BITS: u32 = NUM_OF_BITS as u32;
    const MASK: u32 = ( (1 << NUM_OF_BITS ) -1 ) as u32;
}


#[derive(Default, Clone, Debug)]
pub struct BitData<const N: usize> {
    len: u32,
    mask: u32,
    shift: u8,
}

impl<const N: usize> BitData<N> {
    pub fn new(len: usize) -> Self {
        let (mask, shift) = Self::get_mask_shift(len);
        Self {
            len: len as u32,
            mask,
            shift,
        }
    }

    #[inline]
    pub fn split_index(&self, _idx: usize) -> (usize, usize) {
        panic!()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    #[inline]
    pub fn value_array_size() -> usize {
        1 << N
    }

    #[inline]
    pub fn node_array_size() -> usize {
        N
    }

    #[inline]
    pub fn mask() -> usize {
        Self::value_array_size() - 1
    }

    #[inline]
    fn get_mask_shift(idx: usize) -> (u32, u8) {
        let x = get_highest_bit(idx).expect("no zero value");
        let bucket = x / Self::node_array_size();
        let mask = Self::mask() << bucket;
        (mask as u32, bucket.try_into().unwrap())
    }
}

#[derive(Default, Clone, Debug)]
pub struct NodeArrayData<T: Clone, const N: usize> {
    nodes: ThinVec<Arc<TreeNode<T, N>>>,
    bit_data: BitData<N>,
}

impl<T: Clone, const N: usize> NodeArrayData<T, N> {
    pub fn from(data: &[TreeNode<T, N>]) -> Self {
        assert!(data.len() <= BitData::<N>::node_array_size());
        let len = data.iter().fold(0, |c, v| c + v.len()) as u32;
        let nodes: ThinVec<_> = data
            .into_iter()
            .map(|node| Arc::from(node.clone()))
            .collect();

        Self {
            bit_data: BitData::new(len as usize),
            nodes,
        }
    }

    pub fn len(&self) -> usize {
        self.bit_data.len()
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        assert!(idx <= self.len());
        let (bucket, remainder) = self.bit_data.split_index(idx);
        self.nodes[bucket].get(remainder)
    }
}

#[derive(Default, Clone, Debug)]
pub enum TreeNode<T: Clone, const N: usize> {
    NodeArray(Box<NodeArrayData<T, N>>),
    #[default]
    Empty,
    ValueArray(ThinVec<T>),
}

pub fn align(i: usize, alignment: usize) -> usize {
    let rem = i % alignment;
    if i == 0 {
        i
    } else {
        i + (alignment - rem)
    }
}

impl<T: Clone, const N: usize> TreeNode<T, N> {
    #[inline]

    pub fn get(&self, idx: usize) -> Option<&T> {
        use TreeNode::*;
        match self {
            Empty => None,
            ValueArray(arr) => arr.get(idx),
            NodeArray(node_data) => node_data.get(idx),
        }
    }

    pub fn len(&self) -> usize {
        use TreeNode::*;
        match self {
            Empty => 0,
            ValueArray(arr) => arr.len(),
            NodeArray(node_data) => node_data.len(),
        }
    }

    fn new_tree_root(data: &[Arc<Self>]) -> ThinVec<Arc<Self>> {
        assert!(data.len() <= 5);
        data.iter().cloned().collect()
    }

    fn mk_value_array(data: &[T]) -> Self {
        let mut arr = ThinVec::with_capacity(32);
        arr.extend_from_slice(data);
        Self::ValueArray(arr)
    }

    fn mk_node_array(data: &[TreeNode<T, N>]) -> Self {
        let data = NodeArrayData::from(data);
        Self::NodeArray(data.into())
    }

    pub fn new(data: &[T]) -> Self {
        let values_per_array = BitData::<N>::value_array_size();
        let leafs_per_node = BitData::<N>::node_array_size();
        println!("leafs: {}", leafs_per_node);

        match data.len() {
            0 => Self::Empty,
            _ => {
                if data.len() <= values_per_array {
                    Self::mk_value_array(data)
                } else {
                    let mut to_place: ThinVec<TreeNode<T, N>> = data
                        .chunks(values_per_array)
                        .map(Self::mk_value_array)
                        .collect();

                    while to_place.len() > 1 {
                        let new_chunks: ThinVec<Self> = to_place
                            .chunks(leafs_per_node)
                            .map(Self::mk_node_array)
                            .collect();
                        to_place = new_chunks;
                    }

                    to_place.into_iter().next().unwrap()
                }
            }
        }
    }
}
#[derive(Debug)]
pub struct PVec<T: Clone, const N: usize = 5> {
    node: TreeNode<T, N>,
    len: usize,
    tail_array: ThinVec<T>,
}

impl<T: Clone, const N: usize> Default for PVec<T, N> {
    fn default() -> Self {
        Self {
            node: Default::default(),
            tail_array: ThinVec::with_capacity(32),
            len: 0,
        }
    }
}

trait VecTest<T: Clone> {
    fn from(data: &[T]) -> Self;
    fn get(&self, idx: usize) -> Option<&T>;
}

impl<T: Clone, const N: usize> PVec<T, N> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from(data: &[T]) -> Self {
        let values_per_array = BitData::<N>::value_array_size();

        let elems_to_ingest = data.len();

        let as_nodes = data.len() / values_per_array;

        let node = if as_nodes == 0 {
            TreeNode::Empty
        } else {
            TreeNode::new(&data[0..as_nodes * values_per_array])
        };

        Self {
            len: elems_to_ingest,
            tail_array: data[as_nodes * values_per_array..].into(),
            node,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn get(&self, idx: usize) -> Option<&T> {
        if idx >= self.len() {
            None
        } else {
            match &self.node {
                TreeNode::Empty => self.tail_array.get(idx),

                TreeNode::ValueArray(arr) => {
                    if idx < arr.len() {
                        arr.get(idx)
                    } else {
                        self.tail_array.get(idx - arr.len())
                    }
                }

                TreeNode::NodeArray { .. } => {
                    let capacity = self.len() - self.tail_array.len();

                    if idx >= capacity {
                        self.tail_array.get(idx - capacity)
                    } else {
                        self.node.get(idx)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_it() {
        use std::mem::size_of;

        println!("Node size : {}", size_of::<TreeNode<i32, 5>>());
        println!(
            "Node array data size : {}",
            size_of::<NodeArrayData<i32, 5>>()
        );

        let mut x = vec![];

        for i in 0..164 {
            x.push(i)
        }

        let v: PVec<i32> = PVec::from(&x);

        for idx in 0..v.len() {
            println!("{idx}: {:?} {:?} ", x.get(idx), v.get(idx),)
        }

        println!("{:#?}", v);
        panic!("All done")
    }
}
