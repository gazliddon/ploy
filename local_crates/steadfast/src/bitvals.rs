pub trait BitValuesTrait {
    const BITS: usize;
    const MASK: usize;
    const SIZE: usize;
}

#[derive(Default,Clone)]
pub struct BitValues<const VAL: usize>;

impl<const VAL: usize> BitValuesTrait for BitValues<VAL> {
    const SIZE: usize = if VAL.count_ones() != 1 {
        panic!("Value must be a power of two")
    } else if VAL >= 1 << 32 {
        panic!("Value must be between 1 and 1 << 32")
    } else {
        VAL
    };

    const BITS: usize = ( usize::BITS - VAL.leading_zeros() ) as usize;
    const MASK: usize = (1 << Self::BITS) - 1;
}
